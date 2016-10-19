// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::str;
use std::collections::HashMap;

use super::{
    AttributeId,
    AttributeValue,
    Document,
    ElementId,
    Error,
    FromStream,
    Node,
    NodeType,
    ParseOptions,
    ValueId,
};
use types::{
    Color,
    Transform,
    Length,
    LengthUnit
};
use types::path;

use svgparser::{
    AttributeValue as ParserAttributeValue,
    PaintFallback,
    Stream,
};
use svgparser::svg;
use svgparser::style;

struct CssData<'a> {
    by_tag: HashMap<ElementId, Stream<'a>>,
    by_class: HashMap<&'a [u8], Stream<'a>>,
}

struct NodeTextData<'a> {
    node: Node,
    stream: Stream<'a>,
}

struct LinkData<'a> {
    attr_id: AttributeId,
    iri: &'a [u8],
    fallback: Option<PaintFallback>,
    node: Node,
}

struct Links<'a> {
    // List of unresolved IRI.
    list: Vec<LinkData<'a>>,
    // Store all nodes with id's.
    elems_with_id: HashMap<&'a [u8], Node>,
}

impl<'a> Links<'a> {
    fn append(&mut self, id: AttributeId, iri: &'a [u8], fallback: Option<PaintFallback>,
              node: &Node) {
        self.list.push(LinkData {
            attr_id: id,
            iri: iri,
            fallback: fallback,
            node: node.clone(),
        });
    }
}

type Entities<'a> = HashMap<&'a [u8], &'a [u8]>;

struct PostData<'a> {
    css: CssData<'a>,
    links: Links<'a>,
    entitis: Entities<'a>,
    // List of element with 'class' attribute.
    // We can't process it inplace, because styles can be set after usage.
    classes: Vec<NodeTextData<'a>>,
    // List of style attributes.
    styles: Vec<NodeTextData<'a>>,
}

macro_rules! u8_to_string {
    ($text:expr) => (String::from_utf8_lossy($text).as_ref())
}

pub fn parse_svg(data: &[u8], opt: &ParseOptions) -> Result<Document, Error> {
    let doc = Document::new();
    let mut parent = doc.root();

    let mut tokenizer = svg::Tokenizer::new(data);

    // Since we not only parsing, but also converting an SVG structure,
    // we can't do everything in one take.
    // At first, we create nodes structure with attributes.
    // Than apply CSS. And then ungroup style attributes.
    // Order is important, otherwise we get rendering error.
    let mut post_data = PostData {
        css: CssData {
            by_tag: HashMap::new(),
            by_class: HashMap::new(),
        },
        links: Links {
            list: Vec::new(),
            elems_with_id: HashMap::new(),
        },
        entitis: HashMap::new(),
        classes: Vec::new(),
        styles: Vec::new(),
    };

    // process SVG tokens
    let mut node: Option<Node> = None;
    while let Some(item) = tokenizer.next() {
        match item {
            Ok(t) => try!(process_token(&doc, t, &mut tokenizer,
                                        &mut node, &mut parent,
                                        &mut post_data, &opt)),
            Err(e) => return Err(Error::ParseError(e)),
        }
    }

    // document must contain any children
    if !doc.root().has_children() {
        return Err(Error::EmptyDocument);
    }

    // first element must be an 'svg'
    match doc.children().svg().nth(0) {
        Some(n) => {
            if !n.is_tag_id(ElementId::Svg) {
                return Err(Error::NoSvgElement);
            }
        }
        None => {
            return Err(Error::NoSvgElement);
        }
    }

    try!(resolve_css(&doc, &mut post_data, &opt));

    // resolve styles
    for d in &post_data.styles {
        try!(parse_style_attribute(&d.node, d.stream.clone(), &mut post_data.links,
                                   &post_data.entitis, &opt));
    }

    try!(resolve_links(&post_data.links));

    Ok(doc)
}

fn process_token<'a>(doc: &Document,
                     token: svg::Token<'a>,
                     tokenizer: &mut svg::Tokenizer<'a>,
                     node: &mut Option<Node>,
                     parent: &mut Node,
                     post_data: &mut PostData<'a>,
                     opt: &ParseOptions)
                     -> Result<(), Error> {

    macro_rules! create_node {
        ($nodetype:expr, $buf:expr) => ({
            let e = doc.create_node($nodetype, u8_to_str!($buf));
            *node = Some(e.clone());
            parent.append(&e);
        })
    }

    match token {
        svg::Token::ElementStart(s) => {
            match ElementId::from_name(u8_to_str!(s)) {
                Some(eid) => {
                    let res = try!(parse_svg_element(&doc, tokenizer, eid,
                                                     &mut post_data.css));

                    if let Some(n) = res {
                        *node = Some(n.clone());
                        parent.append(&n);
                    }
                }
                None => {
                    if !opt.parse_unknown_elements {
                        try!(skip_current_element(tokenizer));
                    } else {
                        // create new node
                        let e = try!(doc.create_nonsvg_element(u8_to_string!(s)));
                        *node = Some(e.clone());
                        parent.append(&e);
                    }
                }
            }
        }
        svg::Token::Attribute(name, val) => {
            let n = node.as_ref().unwrap();
            if n.is_svg_element() {
                try!(parse_attribute(&n,
                                     &name,
                                     &mut val.clone(),
                                     post_data,
                                     &opt));
            } else {
                // TODO: store as &str not String
                if opt.parse_unknown_attributes {
                    n.set_attribute(u8_to_str!(name), u8_to_str!(val.slice()));
                }
            }
        }
        svg::Token::ElementEnd(end) => {
            match end {
                svg::ElementEnd::Empty => {}
                svg::ElementEnd::Close(_) => {
                    if !parent.same_node(&doc.root()) {
                        *parent = parent.parent().unwrap();
                    }
                }
                svg::ElementEnd::Open => {
                    if let Some(ref n) = *node {
                        *parent = n.clone();
                    }
                }
            }
        }
        svg::Token::Text(s) => {
            create_node!(NodeType::Text, s.slice());
        }
        svg::Token::Comment(s) => {
            if opt.parse_comments {
                create_node!(NodeType::Comment, s)
            }
        }
        svg::Token::Cdata(s) => {
            create_node!(NodeType::Cdata, s.slice());
        }
        svg::Token::Declaration(s) => {
            // TODO: check that it UTF-8
            if opt.parse_declarations {
                create_node!(NodeType::Declaration, s);
            }
        }
        svg::Token::Entity(name, value) => {
            // check that ENTITY does not contain an element(s)
            let mut s = value;
            s.skip_spaces();
            if !s.at_end() {
                if s.curr_char_raw() == b'<' {
                    return Err(Error::UnsupportedEntity(s.gen_error_pos()));
                }
            }

            post_data.entitis.insert(name, value.slice());
        }
          svg::Token::Whitespace(_)
        | svg::Token::DtdStart(_)
        | svg::Token::DtdEmpty(_)
        | svg::Token::DtdEnd => {
            // do nothing
        }
    }

    // check for 'svg' element only when we parsing root nodes,
    // which is faster
    if parent.node_type() == NodeType::Root {
        // check that the first element of the doc is 'svg'
        if let Some(n) = doc.children().svg().nth(0) {
            if !n.is_tag_id(ElementId::Svg) {
                return Err(Error::NoSvgElement);
            }
        }
    }

    Ok(())
}

fn parse_svg_element<'a>(doc: &Document,
                         tokenizer: &mut svg::Tokenizer<'a>,
                         id: ElementId,
                         css: &mut CssData<'a>)
                         -> Result<Option<Node>, Error> {
    // We never create 'style' element.
    // If 'style' element is empty - we skip it.
    // If it contains CDATA/CSS - we parse it and store it for future use,
    // but node and it's content doesn't imported to DOM.
    if id == ElementId::Style {
        // TODO: process only style with 'type='text/css'' or no 'type' attr.

        // skip attributes, since we only interested in CDATA.
        while let Some(subitem) = tokenizer.next() {
            match subitem {
                Ok(st) => {
                    match st {
                        svg::Token::Attribute(_, _) => {}
                        svg::Token::ElementEnd(svg::ElementEnd::Empty) => {
                            // if 'style' do not have children - return
                            return Ok(None);
                        }
                        _ => break,
                    }
                }
                Err(e) => {
                    return Err(Error::ParseError(e));
                }
            }
        }

        // TODO: check if two or more style elements can exist and how to
        // process them.

        // 'style' node can contain not only one CDATA block,
        // so we process all of them.
        // Also style node can contain only text.
        while let Some(subitem) = tokenizer.next() {
            match subitem {
                Ok(st) => {
                    match st {
                          svg::Token::Cdata(s)
                        | svg::Token::Text(s) => try!(parse_css(&mut s.clone(), css)),
                        svg::Token::Whitespace(_) => {}
                        _ => break,
                    }
                }
                Err(e) => {
                    return Err(Error::ParseError(e));
                }
            }
        }

        Ok(None)
    } else {
        // create new node
        let e = doc.create_element(id);
        Ok(Some(e.clone()))
    }
}

fn parse_attribute<'a>(node: &Node,
                       name: &'a [u8],
                       stream: &mut Stream<'a>,
                       post_data: &mut PostData<'a>,
                       opt: &ParseOptions)
                       -> Result<(), Error> {
    match AttributeId::from_name(u8_to_str!(name)) {
        Some(id) => {
            match id {
                AttributeId::Id => {
                    node.set_id(u8_to_string!(stream.slice()));
                    post_data.links.elems_with_id.insert(stream.slice(), node.clone());
                }
                AttributeId::Style => {
                    post_data.styles.push(NodeTextData {
                        node: node.clone(),
                        stream: *stream,
                    })
                }
                  AttributeId::Transform
                | AttributeId::GradientTransform
                | AttributeId::PatternTransform => {
                    let ts = try!(Transform::from_stream(stream.clone()));
                    node.set_attribute(id, AttributeValue::Transform(ts));
                }
                AttributeId::D => {
                    let p = try!(path::Path::from_stream(stream.clone()));
                    node.set_attribute(AttributeId::D, AttributeValue::Path(p));
                }
                AttributeId::Class => {
                    post_data.classes.push(NodeTextData {
                        node: node.clone(),
                        stream: *stream,
                    })
                }
                _ => {
                    try!(parse_svg_attribute(&node, id, stream, &mut post_data.links,
                                             &post_data.entitis, opt));
                }
            }
        }
        None => {
            if !opt.parse_unknown_attributes {
                return Ok(());
            }

            let value2;

            if !stream.at_end() && stream.is_char_eq_raw(b'&') {
                stream.advance_raw(1);
                let link = stream.slice_next_raw(stream.len_to_or_end(b';'));

                match post_data.entitis.get(link) {
                    Some(link_value) => value2 = Some(*link_value),
                    None => {
                        println!("Warning: Could not resolve ENTITY: '{}'.", u8_to_str!(link));
                        value2 = None;
                    }
                }
            } else {
                value2 = Some(stream.slice());
            }

            if let Some(val) = value2 {
                node.set_attribute(u8_to_str!(name), u8_to_str!(val));
            }
        }
    }

    Ok(())
}

fn parse_svg_attribute<'a>(node: &Node,
                           id: AttributeId,
                           stream: &mut Stream<'a>,
                           links: &mut Links<'a>,
                           entitis: &Entities<'a>,
                           opt: &ParseOptions)
                           -> Result<(), Error> {
    let tag_id = node.tag_id().unwrap();

    let val = match try!(ParserAttributeValue::from_stream(tag_id, id, stream)) {
        ParserAttributeValue::String(v) => Some(AttributeValue::String(u8_to_str!(v).to_string())),
        ParserAttributeValue::IRI(link) | ParserAttributeValue::FuncIRI(link) => {
            // collect links for later processing
            links.append(id, link, None, node);
            None
        }
        ParserAttributeValue::FuncIRIWithFallback(link, ref fallback) => {
            // collect links for later processing
            links.append(id, link, Some(fallback.clone()), node);
            None
        }
        ParserAttributeValue::Number(v) => Some(AttributeValue::Number(v)),
        ParserAttributeValue::NumberList(list) => {
            let mut vec = Vec::new();
            for number in list {
                match number {
                    Ok(n) => vec.push(n),
                    Err(e) => return Err(Error::ParseError(e)),
                }
            }
            Some(AttributeValue::NumberList(vec))
        }
        ParserAttributeValue::Length(v) => {
            Some(AttributeValue::Length(Length::new(v.num, prepare_length(v.unit, opt))))
        }
        ParserAttributeValue::LengthList(list) => {
            let mut vec = Vec::new();
            for number in list {
                match number {
                    Ok(n) => vec.push(Length::new(n.num, prepare_length(n.unit, opt))),
                    Err(e) => return Err(Error::ParseError(e)),
                }
            }
            Some(AttributeValue::LengthList(vec))
        }
        ParserAttributeValue::Color(v) => {
            Some(AttributeValue::Color(Color::new(v.red, v.green, v.blue)))
        }
        ParserAttributeValue::PredefValue(v) => Some(AttributeValue::PredefValue(v)),
        ParserAttributeValue::EntityRef(link) => {
            match entitis.get(link) {
                Some(link_value) => {
                    let mut s = Stream::new(link_value);
                    try!(parse_svg_attribute(node, id, &mut s,
                                             links, entitis, opt));
                    None
                }
                None => {
                    // keep original link
                    let s = format!("&{};", u8_to_str!(link));

                    if link[0] != b'#' {
                        // If link starts with # - than it's probably a Unicode code point.
                        // Otherwise - unknown reference.
                        println!("Warning: Unresolved ENTITY reference: '{}'.", s);
                    }

                    Some(AttributeValue::String(s))
                }
            }
        }
    };

    if let Some(v) = val {
        node.set_attribute(id, v);
    }

    Ok(())
}

fn prepare_length(unit: LengthUnit, opt: &ParseOptions) -> LengthUnit {
    // replace 'px' with none if 'parse_px_unit' option is disabled
    if !opt.parse_px_unit && unit == LengthUnit::Px {
        return LengthUnit::None;
    }

    unit
}

fn parse_css<'a>(stream: &mut Stream<'a>, css: &mut CssData<'a>) -> Result<(), Error> {
    // we support only tiny subset of the CSS2 spec
    fn is_valid_css_char(c: u8) -> bool {
        match c {
            b'#' | b'@' | b':' | b'+' | b'[' | b']' | b'*' | b'>' => false,
            _ => true,
        }
    }

    while !stream.at_end() {
        stream.skip_spaces();

        if stream.at_end() {
            break;
        }

        // skip comments
        if try!(stream.is_char_eq(b'/')) {
            try!(stream.advance(2)); // skip /*
            while !stream.at_end() {
                try!(stream.jump_to(b'*'));
                try!(stream.advance(1));
                if try!(stream.is_char_eq(b'/')) {
                    stream.advance_raw(1);
                    break;
                }
            }
            stream.skip_spaces();
            continue;
        }

        if !is_valid_css_char(try!(stream.curr_char())) {
            return Err(Error::UnsupportedCSS(stream.gen_error_pos()));
        }

        let pos = stream.pos();

        // parse selectors
        let selectors = try!(stream.read_to_trimmed(b'{'));

        if let Some(_) = selectors.iter().position(|c| !is_valid_css_char(*c)) {
            stream.set_pos_raw(pos);
            return Err(Error::UnsupportedCSS(stream.gen_error_pos()));
        }

        stream.advance_raw(1);
        stream.skip_spaces();

        // parse declaration
        let decl_len = try!(stream.len_to(b'}'));
        let substream = Stream::sub_stream(stream, stream.pos(), stream.pos() + decl_len);

        // split selectors
        //
        // we support only tag and class selectors
        let mut s = Stream::new(selectors);
        while !s.at_end() {
            let is_class = s.is_char_eq_raw(b'.');
            if is_class {
                s.advance_raw(1);
            }

            let l = s.len_to_or_end(b',');
            let name = s.read_raw(l);

            if is_class {
                css.by_class.insert(name, substream);
            } else {
                if let Some(eid) = ElementId::from_name(u8_to_str!(name)) {
                    css.by_tag.insert(eid, substream);
                } else {
                    // CSS for a non-SVG element is not supported
                    stream.set_pos_raw(pos);
                    return Err(Error::UnsupportedCSS(stream.gen_error_pos()));
                }
            }

            if !s.at_end() && s.is_char_eq_raw(b',') {
                s.advance_raw(1);
            }
            s.skip_spaces();
        }

        stream.advance_raw(decl_len + 1); // }
        stream.skip_spaces();
    }

    Ok(())
}

fn parse_style_attribute<'a>(node: &Node,
                             stream: Stream<'a>,
                             links: &mut Links<'a>,
                             entitis: &Entities<'a>,
                             opt: &ParseOptions)
                             -> Result<(), Error> {
    let s = style::Tokenizer::new(stream);
    for item in s {
        match item {
            Ok(token) => {
                match token {
                    style::Token::Attribute(name, substream) => {
                        match AttributeId::from_name(u8_to_str!(name)) {
                            Some(id) => {
                                try!(parse_svg_attribute(&node, id, &mut substream.clone(),
                                    links, entitis, opt));
                            }
                            None => {
                                if opt.parse_unknown_attributes {
                                    node.set_attribute(u8_to_str!(name), u8_to_str!(substream.slice()));
                                }
                            }
                        }
                    }
                    style::Token::EntityRef(name) => {
                        if let Some(value) = entitis.get(name) {
                            // TODO: to proper stream
                            let ss = Stream::new(value);
                            try!(parse_style_attribute(&node, ss, links, entitis, opt));
                        }
                    }
                }
            }
            Err(e) => return Err(Error::ParseError(e)),
        }
    }

    Ok(())
}

fn resolve_css<'a>(doc: &Document,
                   post_data: &mut PostData<'a>,
                   opt: &ParseOptions)
                   -> Result<(), Error> {
    for d in &post_data.classes {
        let mut s = d.stream;

        while !s.at_end() {
            s.skip_spaces();
            let len = s.len_to_space_or_end();
            let class = s.read_raw(len);

            match post_data.css.by_class.get(class) {
                Some(stream) => {
                    try!(parse_style_attribute(&d.node, stream.clone(),
                                               &mut post_data.links,
                                               &post_data.entitis, &opt));
                }
                None => {
                    println!("Warning: Could resolve unknown class: {}.",
                             u8_to_str!(class));
                }
            }

            s.skip_spaces();
        }
    }

    for (k, v) in &post_data.css.by_tag {
        for node in doc.descendants().svg() {
            if node.tag_id().unwrap() == *k {
                try!(parse_style_attribute(&node, v.clone(), &mut post_data.links,
                                           &post_data.entitis, &opt));
            }
        }
    }

    Ok(())
}

fn resolve_links(links: &Links) -> Result<(), Error> {
    for d in &links.list {
        match links.elems_with_id.get(d.iri) {
            Some(node) => {
                try!(resolve_link(&d.node, node, d.attr_id, d.iri, &d.fallback));
            }
            None => {
                // check that <paint> contains a fallback value before showing a warning
                match d.fallback {
                    Some(fallback) => {
                        match fallback {
                            PaintFallback::PredefValue(v) => d.node.set_attribute(d.attr_id, v),
                            PaintFallback::Color(c) =>
                                d.node.set_attribute(d.attr_id, Color::new(c.red, c.green, c.blue)),
                        }
                    }
                    None => {
                        match d.attr_id {
                            AttributeId::Filter => {
                                // If an element has a 'filter' attribute with a broken FuncIRI,
                                // then it shouldn't be rendered. But we can't express such behavior
                                // in the svgdom now.
                                // It's not the best solution, but it works.

                                if d.node.is_tag_id(ElementId::Use) {
                                    // TODO: find a solution
                                    // For some reasons if we remove attribute with a broken filter
                                    // from 'use' elements - image will become broken.
                                    // Have no idea why this is happening.
                                    //
                                    // You can test this issue on:
                                    // breeze-icons/icons/actions/22/color-management.svg
                                    return Err(Error::BrokenFuncIri(u8_to_str!(d.iri).to_string()));
                                }

                                if    d.node.parent_element(ElementId::Mask).is_some()
                                   || d.node.parent_element(ElementId::ClipPath).is_some()
                                   || d.node.parent_element(ElementId::Marker).is_some() {
                                    // If our element is inside one of this elements - then do nothing.
                                    // I can't find explanation of this in the SVG spec, but it works.
                                    // Probably because this elements only care about a shape,
                                    // not a style.
                                    println!("Warning: Could not resolve IRI reference: {}.",
                                             u8_to_str!(d.iri));
                                } else {
                                    // Imitate invisible element.
                                    println!("Warning: Unresolved 'filter' link: '{}'. \
                                              Marking the element as invisible.",
                                             u8_to_str!(d.iri));
                                    d.node.set_attribute(AttributeId::Visibility, ValueId::Hidden);
                                }
                            }
                            AttributeId::Fill => {
                                println!("Warning: Could not resolve the 'fill' IRI reference '{}'. \
                                          Fallback to 'none'.",
                                         u8_to_str!(d.iri));
                                d.node.set_attribute(AttributeId::Fill, ValueId::None);
                            }
                            _ => {
                                println!("Warning: Could not resolve IRI reference: {}.",
                                         u8_to_str!(d.iri));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn resolve_link(node: &Node,
                ref_node: &Node,
                aid: AttributeId,
                iri: &[u8],
                fallback: &Option<PaintFallback>)
                -> Result<(), Error> {
    // The SVG uses a fallback paint value not only when the FuncIRI is invalid, but also when
    // a referenced element is invalid. And we don't now is it invalid or not.
    // It will take tonnes of code to validate all supported referenced elements.
    // So we just show an error.
    match *fallback {
        Some(_) => return Err(Error::UnsupportedPaintFallback(u8_to_str!(iri).to_string())),
        None => try!(node.set_link_attribute(aid, ref_node.clone())),
    }
    Ok(())
}

fn skip_current_element(p: &mut svg::Tokenizer) -> Result<(), Error> {
    let mut local_depth = 0;
    for subitem in p {
        match subitem {
            Ok(st) => {
                if let svg::Token::ElementEnd(end) = st {
                    match end {
                        svg::ElementEnd::Empty => {
                            if local_depth == 0 {
                                break;
                            }
                        }
                        svg::ElementEnd::Close(_) => {
                            local_depth -= 1;
                            if local_depth == 0 {
                                break;
                            }
                        }
                        svg::ElementEnd::Open => {
                            local_depth += 1;
                        }
                    }
                }
            }
            Err(e) => {
                return Err(Error::ParseError(e));
            }
        }
    }

    Ok(())
}
