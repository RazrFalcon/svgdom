// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// TODO: this module needs a lot of refactoring.

use std::cmp;
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
    TagName,
    ValueId,
};
use types::{Color, Transform, Length, LengthUnit};
use types::path;

use svgparser::{
    AttributeValue as ParserAttributeValue,
    PaintFallback,
    Stream,
};
use svgparser::svg;
use svgparser::style;

enum Selector<'a> {
    Tag(&'a [u8]),
    Class(&'a [u8]),
}

struct CssData<'a> {
    by_tag: HashMap<String, Stream<'a>>,
    by_class: HashMap<&'a [u8], Stream<'a>>,
}

struct NodeTextData<'a> {
    node: Node,
    stream: Stream<'a>,
}

struct PostAttributes<'a> {
    // List of element with 'class' attribute.
    // We can't process it inplace, because style can be set after usage.
    classes: Vec<NodeTextData<'a>>,
    // List of style attributes.
    styles: Vec<NodeTextData<'a>>,
}

struct LinkData<'a> {
    attr_id: AttributeId,
    iri: &'a [u8],
    fallback: Option<PaintFallback>,
    node: Node,
}

struct PostLinkData<'a> {
    links: Vec<LinkData<'a>>,
    elems_with_id: HashMap<&'a [u8], Node>,
}

type Entities<'a> = HashMap<&'a [u8], &'a [u8]>;

macro_rules! u8_to_string {
    ($text:expr) => (String::from_utf8_lossy($text).into_owned())
}

// Since we not only parsing, but also converting SVG structure,
// we can't do everything in one take.
// At first we create node structure with attributes.
// Than apply CSS. And then ungroup style attributes.
// Order is importtan, otherwise we get rendering error.
pub fn parse_svg(data: &[u8], opt: &ParseOptions) -> Result<Document, Error> {
    let doc = Document::new();
    let mut parent = doc.root();

    let mut p = svg::Tokenizer::new(data);

    let mut is_first_element = true;

    let mut post_link_data = PostLinkData {
        // List of unresolved IRI.
        links: Vec::new(),
        // Store all nodes with id's.
        elems_with_id: HashMap::new(),
    };

    let mut post_attrs = PostAttributes {
        classes: Vec::new(),
        styles: Vec::new(),
    };

    // Map of ENTITY values.
    let mut entitis = HashMap::new();

    // CSS content.
    let mut css = CssData {
        by_tag: HashMap::new(),
        by_class: HashMap::new(),
    };

    let mut node: Option<Node> = None;
    while let Some(item) = p.next() {
        match item {
            Ok(t) => {
                match t {
                    svg::Token::ElementStart(s) => {
                        match ElementId::from_name(u8_to_str!(s)) {
                            Some(id) => {
                                // first element must be <svg>
                                if is_first_element && id != ElementId::Svg {
                                    return Err(Error::NoSvgElement);
                                }

                                if opt.skip_svg_elements.iter().any(|ref x| **x == id) {
                                    try!(skip_current_element(&mut p));
                                    continue;
                                }

                                // We never create 'style' element.
                                // If 'style' element is empty - we skip it.
                                // If it contains CDATA/CSS - we parse it and store it for future use,
                                // but node and it's content doesn't imported to DOM.
                                if id == ElementId::Style {
                                    // TODO: process only style with 'type='text/css'' or no 'type' attr.

                                    // skip attributes, since we only interested in CDATA.
                                    while let Some(subitem) = p.next() {
                                        match subitem {
                                            Ok(st) => {
                                                match st {
                                                    svg::Token::Attribute(_, _) => {}
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
                                    while let Some(subitem) = p.next() {
                                        match subitem {
                                            Ok(st) => {
                                                // let parent = StreamRef {
                                                //     text: data,
                                                //     pos: p.pos(),
                                                // };

                                                match st {
                                                    svg::Token::Cdata(s) => try!(parse_css(&mut s.clone(), &mut css)),
                                                    svg::Token::Text(s) => try!(parse_css(&mut s.clone(), &mut css)),
                                                    svg::Token::Whitespace(_) => {}
                                                    _ => break,
                                                }
                                            }
                                            Err(e) => {
                                                return Err(Error::ParseError(e));
                                            }
                                        }
                                    }

                                    // skip </style>
                                    try!(p.parse_next());
                                } else {
                                    // create new node
                                    let e = doc.create_element(TagName::Id(id));
                                    node = Some(e.clone());
                                    parent.append(&e);
                                }
                            }
                            None => {
                                // first element must be <svg>
                                if is_first_element {
                                    return Err(Error::NoSvgElement);
                                }

                                if !opt.parse_unknown_elements {
                                    try!(skip_current_element(&mut p));
                                } else {
                                    // create new node
                                    let e = doc.create_element(TagName::Name(u8_to_string!(s)));
                                    node = Some(e.clone());
                                    parent.append(&e);
                                }
                            }
                        }

                        is_first_element = false;
                    }
                    svg::Token::Attribute(name, val) => {
                        let tag_name;
                        {
                            let n = node.as_ref().unwrap();
                            tag_name = n.tag_name().unwrap().clone();
                        }

                        match tag_name {
                            TagName::Id(_) => {
                                let n = node.as_ref().unwrap();
                                try!(parse_attribute(&n,
                                                     &name,
                                                     &mut val.clone(),
                                                     &mut post_link_data,
                                                     &mut post_attrs,
                                                     &mut entitis,
                                                     &opt));
                            }
                            TagName::Name(_) => {
                                // we keep all attributes from unknown elements as external
                                let n = node.as_ref().unwrap();
                                n.unknown_attributes_mut()
                                    .insert(u8_to_str!(name).to_string(),
                                            u8_to_str!(val.slice()).to_string());
                            }
                        }
                    }
                    svg::Token::ElementEnd(end) => {
                        match end {
                            svg::ElementEnd::Empty => {}
                            svg::ElementEnd::Close(_) => {
                                if !parent.same_node(&doc.root()) {
                                    parent = parent.parent().unwrap();
                                }
                            }
                            svg::ElementEnd::Open => {
                                match node {
                                    Some(ref n) => parent = n.clone(),
                                    None => {}
                                }
                            }
                        }
                    }
                    svg::Token::Text(s) => {
                        let e = doc.create_node(NodeType::Text, u8_to_str!(s.slice()));
                        node = Some(e.clone());
                        parent.append(&e);
                    }
                    svg::Token::Comment(s) => {
                        if opt.parse_comments {
                            let e = doc.create_node(NodeType::Comment, u8_to_str!(s));
                            node = Some(e.clone());
                            parent.append(&e);
                        }
                    }
                    svg::Token::Cdata(s) => {
                        let e = doc.create_node(NodeType::Cdata, u8_to_str!(s.slice()));
                        node = Some(e.clone());
                        parent.append(&e);
                    }
                    svg::Token::Whitespace(_) => {
                        // do nothing
                    }
                    svg::Token::Declaration(s) => {
                        // TODO: check that it UTF-8
                        if opt.parse_declarations {
                            let e = doc.create_node(NodeType::Declaration, u8_to_str!(s));
                            node = Some(e.clone());
                            parent.append(&e);
                        }
                    }
                    svg::Token::DtdStart(_) => {
                        // do nothing
                    }
                    svg::Token::DtdEmpty(_) => {
                        // do nothing
                    }
                    svg::Token::Entity(name, value) => {
                        let mut s = value.clone();
                        s.skip_spaces();
                        if !s.at_end() {
                            if s.curr_char_raw() == b'<' {
                                return Err(Error::UnsupportedEntity(s.gen_error_pos()));
                            }
                        }
                        entitis.insert(name, value.slice());
                    }
                    svg::Token::DtdEnd => {
                        // do nothing
                    }
                }
            }
            Err(e) => {
                return Err(Error::ParseError(e));
            }
        }
    }

    if !doc.root().has_children() {
        return Err(Error::EmptyDocument);
    }

    if !doc.root().has_child_with_tag_name(&TagName::Id(ElementId::Svg)) {
        return Err(Error::NoSvgElement);
    }

    for d in post_attrs.classes {
        let mut s = d.stream;

        while !s.at_end() {
            s.skip_spaces();
            let len = s.len_to_space_or_end();
            let class = s.read_raw(len);

            match css.by_class.get(class) {
                Some(stream) => {
                    try!(parse_style_attribute(&d.node, stream.clone(), &mut post_link_data, &entitis, &opt));
                }
                None => {
                    println!("Warning: Could resolve unknown class: {}.",
                             u8_to_str!(class));
                }
            }

            s.skip_spaces();
        }
    }

    for (k, v) in &css.by_tag {
        for node in doc.root().descendants() {
            let mut is_valid_tag = false;
            match node.tag_name() {
                Some(ref tag_name) => {
                    let str_name = match &**tag_name {
                        &TagName::Id(ref id) => id.name().to_string(),
                        &TagName::Name(ref name) => name.clone(),
                    };
                    if str_name == *k {
                        is_valid_tag = true;
                    }
                }
                None => {}
            }

            if is_valid_tag {
                try!(parse_style_attribute(&node, v.clone(), &mut post_link_data, &entitis, &opt));
            }
        }
    }

    for ref d in post_attrs.styles {
        try!(parse_style_attribute(&d.node, d.stream.clone(), &mut post_link_data, &entitis, &opt));
    }

    try!(resolve_links(&post_link_data));

    return Ok(doc);
}

fn parse_attribute<'a>(node: &Node,
                       name: &'a [u8],
                       mut stream: &mut Stream<'a>,
                       mut post_link_data: &mut PostLinkData<'a>,
                       post_attrs: &mut PostAttributes<'a>,
                       entitis: &mut Entities<'a>,
                       opt: &ParseOptions)
                       -> Result<(), Error> {
    match AttributeId::from_name(u8_to_str!(name)) {
        Some(id) => {
            match id {
                AttributeId::Id => {
                    // TODO: check that id is ascii
                    node.set_id(u8_to_string!(stream.slice()));
                    post_link_data.elems_with_id.insert(stream.slice(), node.clone());
                }
                AttributeId::Style => {
                    post_attrs.styles.push(NodeTextData {
                        node: node.clone(),
                        stream: stream.clone(),
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
                    post_attrs.classes.push(NodeTextData {
                        node: node.clone(),
                        stream: stream.clone(),
                    })
                }
                _ => {
                    try!(parse_svg_attribute(&node, id, &mut stream, post_link_data, entitis, opt));
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
                let link = stream.slice_next_raw(stream.len_to_char_or_end(b';'));

                match entitis.get(link) {
                    Some(link_value) => value2 = Some(*link_value),
                    None => {
                        println!("Warning: Could not resolve ENTITY: '{}'.", u8_to_str!(link));
                        value2 = None;
                    }
                }
            } else {
                value2 = Some(stream.slice());
            }

            match value2 {
                Some(val) => {
                    node.unknown_attributes_mut()
                        .insert(u8_to_str!(name).to_string(),
                                u8_to_str!(val).to_string());
                }
                None => {
                    // TODO: show error
                }
            }
        }
    }

    Ok(())
}

fn parse_svg_attribute<'a>(node: &Node,
                           id: AttributeId,
                           mut stream: &mut Stream<'a>,
                           post_link_data: &mut PostLinkData<'a>,
                           entitis: &Entities<'a>,
                           opt: &ParseOptions)
                           -> Result<(), Error> {
    let tag_id = node.tag_id().unwrap();

    let val = try!(ParserAttributeValue::from_stream(tag_id, id, &mut stream));
    let val2 = match val {
        ParserAttributeValue::String(v) => Some(AttributeValue::String(u8_to_str!(v).to_string())),
          ParserAttributeValue::IRI(link)
        | ParserAttributeValue::FuncIRI(link) => {
            try!(process_link(link, None, id, node, post_link_data));
            None
        }
        ParserAttributeValue::FuncIRIWithFallback(link, ref fallback) => {
            try!(process_link(link, Some(fallback.clone()), id, node, post_link_data));
            None
        }
        ParserAttributeValue::Number(v) => Some(AttributeValue::Number(v)),
        ParserAttributeValue::NumberList(mut list) => {
            let mut vec = Vec::new();
            while let Some(number) = list.next() {
                match number {
                    Ok(n) => vec.push(n),
                    Err(e) => return Err(Error::ParseError(e)),
                }
            }
            Some(AttributeValue::NumberList(vec))
        }
        ParserAttributeValue::Length(v) => {
            if opt.parse_px_unit {
                Some(AttributeValue::Length(Length::new(v.num, v.unit)))
            } else {
                if v.unit == LengthUnit::Px {
                    let mut newv = v.clone();
                    newv.unit = LengthUnit::None;
                    Some(AttributeValue::Length(Length::new(newv.num, newv.unit)))
                } else {
                    Some(AttributeValue::Length(Length::new(v.num, v.unit)))
                }
            }
        }
        ParserAttributeValue::LengthList(mut list) => {
            let mut vec = Vec::new();
            while let Some(number) = list.next() {
                match number {
                    Ok(n) => vec.push(Length::new(n.num, n.unit)),
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
            // NOTE: We store an entity ref's value only as string.
            // If we will have link to path of transform - they will be ignored.
            // This is, probably, bad.

            // TODO: to func and join with parse_attribute()
            match entitis.get(link) {
                Some(link_value) => Some(AttributeValue::String(u8_to_str!(link_value).to_string())),
                None => {
                    let mut s = String::new();
                    s.push('&');
                    s.push_str(&u8_to_str!(link).to_string());
                    s.push(';');

                    Some(AttributeValue::String(s))
                }
            }
        }
    };

    match val2 {
        Some(v) => node.set_attribute(id, v),
        None => {},
    }

    Ok(())
}

fn process_link<'a>(iri: &'a [u8],
                    fallback: Option<PaintFallback>,
                    aid: AttributeId,
                    node: &Node,
                    post_link_data: &mut PostLinkData<'a>)
                    -> Result<(), Error> {
    match post_link_data.elems_with_id.get(iri) {
        Some(link_node) => {
            try!(resolve_link(node, link_node, aid, iri, &fallback));
        }
        None => {
            // If linked element is not found, keep this IRI until we finish
            // parsing of the whole doc. Since IRI can reference elements in any order
            // and we just not parsed this element yet.
            // Then we can check again.
            post_link_data.links.push(LinkData {
                attr_id: aid,
                iri: iri,
                fallback: fallback,
                node: node.clone(),
            });
        }
    }

    Ok(())
}

fn parse_css<'a>(stream: &mut Stream<'a>, css: &mut CssData<'a>) -> Result<(), Error> {
    while !stream.at_end() {
        stream.skip_spaces();

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

        let selector: Selector;

        match try!(stream.curr_char()) {
            b'.' => {
                stream.advance_raw(1);
                let len = cmp::min(stream.len_to_char_or_end(b'{'), stream.len_to_space_or_end());
                let class = stream.read_raw(len);
                selector = Selector::Class(class);
            }
            b'#' | b'@' | b':' => {
                return Err(Error::UnsupportedCSS(stream.gen_error_pos()));
            }
            _ => {
                let len = cmp::min(stream.len_to_char_or_end(b'{'), stream.len_to_space_or_end());
                let tag = stream.read_raw(len);
                selector = Selector::Tag(tag);
            }
        }

        try!(stream.jump_to(b'{'));

        if !try!(stream.is_char_eq(b'{')) {
            return Err(Error::InvalidCSS(stream.gen_error_pos()));
        }
        try!(stream.consume_char(b'{'));

        let end = stream.pos() + stream.len_to_char_or_end(b'}');
        let substream = Stream::sub_stream(&stream, stream.pos(), end);
        try!(stream.advance(substream.left()));

        try!(stream.consume_char(b'}'));

        match selector {
            Selector::Class(class) => {
                css.by_class.insert(class, substream);
            }
            Selector::Tag(tag) => {
                css.by_tag.insert(u8_to_string!(tag), substream);
            }
        }

        stream.skip_spaces();
    }

    Ok(())
}

fn parse_style_attribute<'a>(node: &Node,
                             stream: Stream<'a>,
                             post_link_data: &mut PostLinkData<'a>,
                             entitis: &Entities<'a>,
                             opt: &ParseOptions)
                             -> Result<(), Error> {
    let mut s = style::Tokenizer::new(stream);
    while let Some(item) = s.next() {
        match item {
            Ok(token) => {
                match token {
                    style::Token::Attribute(name, substream) => {
                        match AttributeId::from_name(u8_to_str!(name)) {
                            Some(id) => {
                                try!(parse_svg_attribute(&node, id, &mut substream.clone(),
                                    post_link_data, entitis, opt));
                            }
                            None => {
                                // TODO: maybe do not skip?
                                println!("Warning: Unknown style attr: '{}'.", u8_to_str!(name));
                            }
                        }
                    }
                    style::Token::EntityRef(name) => {
                        match entitis.get(name) {
                            Some(value) => {
                                // TODO: to proper stream
                                let ss = Stream::new(value);
                                try!(parse_style_attribute(&node, ss, post_link_data, entitis, opt));
                            }
                            None => {}
                        }
                    }
                }
            }
            Err(e) => return Err(Error::ParseError(e)),
        }
    }

    Ok(())
}

fn resolve_links(post_link_data: &PostLinkData) -> Result<(), Error> {
    for d in &post_link_data.links {
        match post_link_data.elems_with_id.get(d.iri) {
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
                        if d.attr_id == AttributeId::Filter {
                            // If an element has a 'filter' attribute with broken FuncIRI,
                            // then it shouldn't be rendered. But we can't express such behavior
                            // in the svgdom now.
                            // It's not the best solution, but it works.
                            if    d.node.parent_element(ElementId::Mask).is_some()
                               || d.node.parent_element(ElementId::ClipPath).is_some()
                               || d.node.parent_element(ElementId::Marker).is_some() {
                                // If our element is inside one of this elements - then do nothing.
                                // I can't find explanation of this in the SVG spec, but it works.
                                // Probably because this element only care about a shape,
                                // not a style.
                            } else {
                                // Imitate invisible element.
                                d.node.set_attribute(AttributeId::Visibility, ValueId::Hidden);
                            }
                        }

                        println!("Warning: Could not resolve IRI reference: {}.",
                                 u8_to_str!(d.iri));
                    }
                }
            }
        }
    }

    Ok(())
}

fn resolve_link(node: &Node, ref_node: &Node, aid: AttributeId, iri: &[u8],
                fallback: &Option<PaintFallback>)
                -> Result<(), Error> {
    // The SVG uses a fallback paint value not only when the FuncIRI is invalid, but also when
    // a referenced element is invalid. And we don't now is it invalid or not.
    // It will take tonnes of code to validate all supported referenced elements.
    // So we just show an error.
    match fallback {
        &Some(_) =>
            return Err(Error::UnsupportedPaintFallback(u8_to_str!(iri).to_string())),
        &None =>
            try!(node.set_link_attribute(aid, ref_node.clone())),
    }
    Ok(())
}

fn skip_current_element(p: &mut svg::Tokenizer) -> Result<(), Error> {
    let mut local_depth = 0;
    while let Some(subitem) = p.next() {
        match subitem {
            Ok(st) => {
                match st {
                    svg::Token::ElementStart(_) => {
                    }
                    svg::Token::ElementEnd(end) => {
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
                    _ => {},
                }
            }
            Err(e) => {
                return Err(Error::ParseError(e));
            }
        }
    }

    Ok(())
}
