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
    ErrorPos,
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
    TextFrame,
    Tokenize,
};
use svgparser::svg;
use svgparser::style;

use simplecss;

struct NodeStreamData<'a> {
    node: Node,
    stream: TextFrame<'a>,
}

struct NodeTextData<'a> {
    node: Node,
    text: &'a str,
}

struct LinkData<'a> {
    attr_id: AttributeId,
    iri: &'a str,
    fallback: Option<PaintFallback>,
    node: Node,
}

struct Links<'a> {
    /// List of all parsed IRI and FuncIRI.
    list: Vec<LinkData<'a>>,
    /// Store all nodes with id's.
    ///
    /// For performance reasons only.
    elems_with_id: HashMap<&'a str, Node>,
}

impl<'a> Links<'a> {
    fn append(&mut self, id: AttributeId, iri: &'a str, fallback: Option<PaintFallback>,
              node: &Node) {
        self.list.push(LinkData {
            attr_id: id,
            iri: iri,
            fallback: fallback,
            node: node.clone(),
        });
    }
}

// TODO: to TextFrame
type Entities<'a> = HashMap<&'a str, &'a str>;

struct PostData<'a> {
    css_list: Vec<TextFrame<'a>>,
    links: Links<'a>,
    entitis: Entities<'a>,
    // List of element with 'class' attribute.
    // We can't process it inplace, because styles can be set after usage.
    class_attrs: Vec<NodeTextData<'a>>,
    // List of style attributes.
    style_attrs: Vec<NodeStreamData<'a>>,
}

pub fn parse_svg(text: &str, opt: &ParseOptions) -> Result<Document, Error> {
    let doc = Document::new();
    let mut parent = doc.root();

    let mut tokenizer = svg::Tokenizer::from_str(text);

    // Since we not only parsing, but also converting an SVG structure,
    // we can't do everything in one take.
    // At first, we create nodes structure with attributes.
    // Than apply CSS. And then ungroup style attributes.
    // Order is important, otherwise we get rendering error.
    let mut post_data = PostData {
        css_list: Vec::new(),
        links: Links {
            list: Vec::new(),
            elems_with_id: HashMap::new(),
        },
        entitis: HashMap::new(),
        class_attrs: Vec::new(),
        style_attrs: Vec::new(),
    };

    // process SVG tokens
    let mut node: Option<Node> = None;

    loop {
        let t = tokenizer.parse_next()?;
        match t {
            svg::Token::EndOfStream => break,
            _ => {
                process_token(&doc, t, &mut tokenizer,
                              &mut node, &mut parent,
                              &mut post_data, &opt)?
            }
        }
    }

    // document must contain any children
    if !doc.root().has_children() {
        return Err(Error::EmptyDocument);
    }

    // first element must be an 'svg'
    match doc.children().svg().nth(0) {
        Some(n) => {
            if !n.is_tag_name(ElementId::Svg) {
                return Err(Error::NoSvgElement);
            }
        }
        None => {
            return Err(Error::NoSvgElement);
        }
    }

    resolve_css(&doc, &mut post_data, &opt)?;

    // resolve styles
    for d in &post_data.style_attrs {
        parse_style_attribute(&d.node, d.stream, &mut post_data.links,
                              &post_data.entitis, &opt)?;
    }

    resolve_links(&post_data.links)?;

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
            let e = doc.create_node($nodetype, $buf);
            *node = Some(e.clone());
            parent.append(&e);
        })
    }

    match token {
        svg::Token::ElementStart(s) => {
            match ElementId::from_name(s) {
                Some(eid) => {
                    let res = parse_svg_element(&doc, tokenizer, eid, &mut post_data.css_list)?;

                    if let Some(n) = res {
                        *node = Some(n.clone());
                        parent.append(&n);
                    }
                }
                None => {
                    if !opt.parse_unknown_elements {
                        skip_current_element(tokenizer)?;
                    } else {
                        // create new node
                        let e = doc.create_element(s);
                        *node = Some(e.clone());
                        parent.append(&e);
                    }
                }
            }
        }
        svg::Token::Attribute(name, val) => {
            let n = node.as_ref().unwrap();
            if n.is_svg_element() {
                parse_attribute(&n,
                                name,
                                val,
                                post_data,
                                &opt)?;
            } else {
                // TODO: store as &str not String
                if opt.parse_unknown_attributes {
                    n.set_attribute(name, val.slice());
                }
            }
        }
        svg::Token::ElementEnd(end) => {
            match end {
                svg::ElementEnd::Empty => {}
                svg::ElementEnd::Close(_) => {
                    if *parent != doc.root {
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
            let mut s = Stream::from_frame(value);
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
        | svg::Token::DtdEnd
        | svg::Token::EndOfStream => {
            // do nothing
        }
    }

    // check for 'svg' element only when we parsing root nodes,
    // which is faster
    if parent.node_type() == NodeType::Root {
        // check that the first element of the doc is 'svg'
        if let Some(n) = doc.children().svg().nth(0) {
            if !n.is_tag_name(ElementId::Svg) {
                return Err(Error::NoSvgElement);
            }
        }
    }

    Ok(())
}

fn parse_svg_element<'a>(doc: &Document,
                         tokenizer: &mut svg::Tokenizer<'a>,
                         id: ElementId,
                         styles: &mut Vec<TextFrame<'a>>)
                         -> Result<Option<Node>, Error> {
    // We never create 'style' element.
    // If 'style' element is empty - we skip it.
    // If it contains CDATA/CSS - we parse it and store it for future use,
    // but node and it's content doesn't imported to DOM.
    if id == ElementId::Style {
        // TODO: process only style with 'type='text/css'' or no 'type' attr.

        // skip attributes, since we only interested in CDATA.
        let mut is_valid_type = true;

        loop {
            match tokenizer.parse_next()? {
                svg::Token::Attribute(name, value) => {
                    if name == "type" {
                        if value.slice() != "text/css" {
                            is_valid_type = false;
                            break;
                        }
                    }
                }
                svg::Token::ElementEnd(svg::ElementEnd::Empty) => {
                    // if 'style' do not have children - return
                    return Ok(None);
                }
                _ => break,
            }
        }

        if !is_valid_type {
            skip_current_element(tokenizer)?;
            return Ok(None);
        }

        // TODO: check if two or more style elements can exist and how to
        // process them.

        // 'style' node can contain not only one CDATA block,
        // so we process all of them.
        // Also style node can contain only text.

        loop {
            match tokenizer.parse_next()? {
                  svg::Token::Cdata(s)
                | svg::Token::Text(s) => styles.push(s),
                svg::Token::Whitespace(_) => {}
                _ => break,
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
                       name: &'a str,
                       value: TextFrame<'a>,
                       post_data: &mut PostData<'a>,
                       opt: &ParseOptions)
                       -> Result<(), Error> {
    match AttributeId::from_name(name) {
        Some(id) => {
            match id {
                AttributeId::Id => {
                    node.set_id(value.slice());
                    post_data.links.elems_with_id.insert(value.slice(), node.clone());
                }
                AttributeId::Style => {
                    // we store 'class' attributes for later use
                    post_data.style_attrs.push(NodeStreamData {
                        node: node.clone(),
                        stream: value,
                    })
                }
                  AttributeId::Transform
                | AttributeId::GradientTransform
                | AttributeId::PatternTransform => {
                    let ts = Transform::from_stream(value)?;
                    if !ts.is_default() {
                        node.set_attribute(id, AttributeValue::Transform(ts));
                    }
                }
                AttributeId::D => {
                    let p = path::Path::from_stream(value.clone())?;
                    node.set_attribute(AttributeId::D, AttributeValue::Path(p));
                }
                AttributeId::Class => {
                    // we store 'class' attributes for later use

                    let mut s = Stream::from_frame(value);
                    while !s.at_end() {
                        s.skip_spaces();

                        let len = s.len_to_space_or_end();
                        let class_raw = s.read_raw(len);
                        let class = class_raw;

                        post_data.class_attrs.push(NodeTextData {
                            node: node.clone(),
                            text: class,
                        });

                        s.skip_spaces();
                    }
                }
                _ => {
                    parse_svg_attribute(&node, id, value, &mut post_data.links,
                                        &post_data.entitis, opt)?;
                }
            }
        }
        None => {
            if !opt.parse_unknown_attributes {
                return Ok(());
            }

            let value2;

            let mut stream = Stream::from_frame(value);
            if !stream.at_end() && stream.is_char_eq_raw(b'&') {
                stream.advance_raw(1);
                let link = stream.slice_next_raw(stream.len_to_or_end(b';'));

                match post_data.entitis.get(link) {
                    Some(link_value) => value2 = Some(*link_value),
                    None => {
                        warnln!("Could not resolve ENTITY: '{}'.", link);
                        value2 = None;
                    }
                }
            } else {
                value2 = Some(stream.slice());
            }

            if let Some(val) = value2 {
                node.set_attribute(name, val);
            }
        }
    }

    Ok(())
}

fn parse_svg_attribute<'a>(node: &Node,
                           id: AttributeId,
                           frame: TextFrame<'a>,
                           links: &mut Links<'a>,
                           entitis: &Entities<'a>,
                           opt: &ParseOptions)
                           -> Result<(), Error> {
    let tag_id = node.tag_id().unwrap();

    let val = match ParserAttributeValue::from_frame(tag_id, id, frame)? {
        ParserAttributeValue::String(v) => {
            Some(AttributeValue::String(v.to_string()))
        }
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
        ParserAttributeValue::Number(v) => {
            Some(AttributeValue::Number(v))
        }
        ParserAttributeValue::NumberList(list) => {
            let mut vec = Vec::new();
            for number in list {
                match number {
                    Ok(n) => vec.push(n),
                    Err(e) => return Err(Error::ParseError(e)),
                }
            }

            if !vec.is_empty() {
                Some(AttributeValue::NumberList(vec))
            } else {
                None
            }
        }
        ParserAttributeValue::Length(v) => {
            Some(AttributeValue::Length(Length::new(v.num, prepare_length_unit(v.unit, opt))))
        }
        ParserAttributeValue::LengthList(list) => {
            let mut vec = Vec::new();
            for number in list {
                match number {
                    Ok(n) => vec.push(Length::new(n.num, prepare_length_unit(n.unit, opt))),
                    Err(e) => return Err(Error::ParseError(e)),
                }
            }

            if !vec.is_empty() {
                Some(AttributeValue::LengthList(vec))
            } else {
                None
            }
        }
        ParserAttributeValue::Color(v) => {
            Some(AttributeValue::Color(Color::new(v.red, v.green, v.blue)))
        }
        ParserAttributeValue::PredefValue(v) => {
            Some(AttributeValue::PredefValue(v))
        }
        ParserAttributeValue::EntityRef(link) => {
            match entitis.get(link) {
                Some(link_value) => {
                    let frame = TextFrame::from_str(link_value);
                    parse_svg_attribute(node, id, frame, links, entitis, opt)?;
                    None
                }
                None => {
                    // keep original link
                    let s = format!("&{};", link);

                    if link.as_bytes()[0] != b'#' {
                        // If link starts with # - than it's probably a Unicode code point.
                        // Otherwise - unknown reference.
                        warnln!("Unresolved ENTITY reference: '{}'.", s);
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

fn prepare_length_unit(unit: LengthUnit, opt: &ParseOptions) -> LengthUnit {
    // replace 'px' with 'none' if 'parse_px_unit' option is disabled
    if !opt.parse_px_unit && unit == LengthUnit::Px {
        return LengthUnit::None;
    }

    unit
}

fn parse_style_attribute<'a>(node: &Node,
                             frame: TextFrame<'a>,
                             links: &mut Links<'a>,
                             entitis: &Entities<'a>,
                             opt: &ParseOptions)
                             -> Result<(), Error> {
    let mut s = style::Tokenizer::from_frame(frame);

    loop {
        match s.parse_next()? {
            style::Token::Attribute(name, attr_frame) => {
                match AttributeId::from_name(name) {
                    Some(id) => {
                        parse_svg_attribute(&node, id, attr_frame,
                                            links, entitis, opt)?;
                    }
                    None => {
                        if opt.parse_unknown_attributes {
                            node.set_attribute(name, attr_frame.slice());
                        }
                    }
                }
            }
            style::Token::EntityRef(name) => {
                if let Some(value) = entitis.get(name) {
                    // TODO: to proper stream
                    let ss = TextFrame::from_str(value);
                    parse_style_attribute(&node, ss, links, entitis, opt)?;
                }
            }
            style::Token::EndOfStream => break,
        }
    }

    Ok(())
}

fn resolve_css<'a>(doc: &Document,
                   post_data: &mut PostData<'a>,
                   opt: &ParseOptions)
                   -> Result<(), Error> {
    use simplecss::Token as CssToken;

    #[derive(Clone,Copy,Debug)]
    enum CssSelector<'a> {
        Universal,
        Type(&'a str),
        Id(&'a str),
        Class(&'a str),
    }

    fn gen_err_pos(frame: TextFrame, pos: usize) -> ErrorPos {
        let mut s = Stream::from_str(frame.full_slice());
        s.set_pos_raw(pos);
        s.gen_error_pos()
    }

    let mut selectors: Vec<CssSelector> = Vec::new();
    let mut values: Vec<(&str,&str)> = Vec::with_capacity(16);

    // remember all resolved classes
    let mut resolved_classes: Vec<&str> = Vec::with_capacity(16);

    // we have to make a copy to allow passing mutable 'post_data'
    // it's cheap, because in a 99.9% cases we have only one style
    let styles = post_data.css_list.clone();

    for style in &styles {
        let mut tokenizer = {
            let mut s = Stream::from_frame(*style);

            // check for a empty string
            s.skip_spaces();
            if s.at_end() {
                // ignore such CSS
                continue;
            }

            let text = style.full_slice();

            // we use 'new_bound' method to get absolute error positions
            simplecss::Tokenizer::new_bound(text, style.start() + s.pos(), style.end())
        };

        'root: loop {
            selectors.clear();
            values.clear();

            // get list of selectors
            loop {
                // remember position before next token
                let last_pos = tokenizer.pos();

                let token = tokenizer.parse_next()?;

                match token {
                    CssToken::EndOfStream => {
                        // parsing finished
                        break 'root;
                    }
                    CssToken::BlockStart => {
                        // stop selectors parsing
                        break;
                    }
                    CssToken::Comma => {
                        // we ignore 'comma' token
                        continue;
                    }
                    _ => {}
                }

                // currently we support only simple selectors
                let selector = match token {
                    CssToken::UniversalSelector     => CssSelector::Universal,
                    CssToken::TypeSelector(name)    => CssSelector::Type(name),
                    CssToken::IdSelector(name)      => CssSelector::Id(name),
                    CssToken::ClassSelector(name)   => CssSelector::Class(name),

                      CssToken::AttributeSelector(_)
                    | CssToken::PseudoClass(_)
                    | CssToken::LangPseudoClass(_)
                    | CssToken::Combinator(_) => {
                        return Err(Error::UnsupportedCSS(gen_err_pos(*style, last_pos)));
                    }
                    _ => {
                        return Err(Error::InvalidCSS(gen_err_pos(*style, last_pos)));
                    }
                };

                selectors.push(selector);
            }

            // get list of declarations
            loop {
                // remember position before next token
                let last_pos = tokenizer.pos();

                match tokenizer.parse_next()? {
                    CssToken::Declaration(name, value) => values.push((name, value)),
                    CssToken::BlockEnd => break,
                    CssToken::EndOfStream => break 'root,
                    _ => {
                        return Err(Error::InvalidCSS(gen_err_pos(*style, last_pos)));
                    }
                }
            }

            // process selectors
            for selector in &selectors {
                match *selector {
                    CssSelector::Universal => {
                        for node in doc.descendants().svg() {
                            apply_css_attributes(&values, &node, &mut post_data.links,
                                                 &post_data.entitis, opt)?;
                        }
                    }
                    CssSelector::Type(name) => {
                        if let Some(eid) = ElementId::from_name(name) {
                            for node in doc.descendants().svg().filter(|n| n.is_tag_name(eid)) {
                                apply_css_attributes(&values, &node, &mut post_data.links,
                                                     &post_data.entitis, opt)?;
                            }
                        } else {
                            warnln!("CSS styles for a non-SVG element ('{}') are ignored.",
                                     name);
                        }
                    }
                    CssSelector::Id(name) => {
                        if let Some(node) = doc.descendants().svg().find(|n| *n.id() == name) {
                            apply_css_attributes(&values, &node, &mut post_data.links,
                                                 &post_data.entitis, opt)?;
                        }
                    }
                    CssSelector::Class(name) => {
                        // we use already collected list of 'class' attributes
                        for d in post_data.class_attrs.iter().filter(|n| n.text == name) {
                            apply_css_attributes(&values, &d.node, &mut post_data.links,
                                                 &post_data.entitis, opt)?;

                            resolved_classes.push(name);
                        }
                    }
                }
            }
        }
    }

    postprocess_class_selector(&resolved_classes, &mut post_data.class_attrs, opt);

    Ok(())
}

fn postprocess_class_selector<'a>(resolved_classes: &Vec<&str>,
                                  class_attrs: &mut Vec<NodeTextData<'a>>,
                                  opt: &ParseOptions) {
    // remove resolved classes
    class_attrs.retain(|n| !resolved_classes.contains(&n.text));

    if opt.skip_unresolved_classes {
        for d in class_attrs {
            warnln!("Could not resolve an unknown class: {}.", d.text);
        }
    } else {
        // create 'class' attributes with unresolved classes
        for d in class_attrs {
            if d.node.has_attribute(AttributeId::Class) {
                let mut attrs = d.node.attributes_mut();
                let mut class_val = attrs.get_value_mut(AttributeId::Class);
                if let Some(&mut AttributeValue::String(ref mut text)) = class_val {
                    text.push(' ');
                    text.push_str(d.text);
                }
            } else {
                d.node.set_attribute(AttributeId::Class, d.text);
            }
        }
    }
}

fn apply_css_attributes<'a>(values: &Vec<(&str,&'a str)>,
                            node: &Node,
                            links: &mut Links<'a>,
                            entitis: &Entities<'a>,
                            opt: &ParseOptions)
                            -> Result<(), Error> {
    for &(aname, avalue) in values {
        match AttributeId::from_name(aname) {
            Some(aid) => {
                // TODO: to a proper stream
                parse_svg_attribute(node, aid, TextFrame::from_str(avalue),
                                    links, entitis, opt)?;
            }
            None => {
                if opt.parse_unknown_attributes {
                    node.set_attribute(aname, avalue);
                }
            }
        }
    }

    Ok(())
}

fn resolve_links(links: &Links) -> Result<(), Error> {
    for d in &links.list {
        match links.elems_with_id.get(d.iri) {
            Some(node) => {
                // The SVG uses a fallback paint value not only when the FuncIRI is invalid,
                // but also when a referenced element is invalid.
                // And we don't know is it invalid or not.
                // It will take tonnes of code to validate all supported referenced elements,
                // so we just show an error.
                match d.fallback {
                    Some(_) => {
                        let s = d.iri.to_string();
                        return Err(Error::UnsupportedPaintFallback(s))
                    }
                    None => d.node.set_link_attribute(d.attr_id, node.clone())?,
                }
            }
            None => {
                // check that <paint> contains a fallback value before showing a warning
                match d.fallback {
                    Some(fallback) => {
                        match fallback {
                            PaintFallback::PredefValue(v) =>
                                d.node.set_attribute(d.attr_id, v),
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

                                if d.node.is_tag_name(ElementId::Use) {
                                    // TODO: find a solution
                                    // For some reasons if we remove attribute with a broken filter
                                    // from 'use' elements - image will become broken.
                                    // Have no idea why this is happening.
                                    //
                                    // You can test this issue on:
                                    // breeze-icons/icons/actions/22/color-management.svg
                                    let s = d.iri.to_string();
                                    return Err(Error::BrokenFuncIri(s));
                                }

                                let flag = d.node.parents().any(|n| {
                                       n.is_tag_name(ElementId::Mask)
                                    || n.is_tag_name(ElementId::ClipPath)
                                    || n.is_tag_name(ElementId::Marker)
                                });

                                if flag {
                                    // If our element is inside one of this elements - then do nothing.
                                    // I can't find explanation of this in the SVG spec, but it works.
                                    // Probably because this elements only care about a shape,
                                    // not a style.
                                    warnln!("Could not resolve IRI reference: {}.", d.iri);
                                } else {
                                    // Imitate invisible element.
                                    warnln!("Unresolved 'filter' link: '{}'. \
                                             Marking the element as invisible.",
                                             d.iri);
                                    d.node.set_attribute(AttributeId::Visibility, ValueId::Hidden);
                                }
                            }
                            AttributeId::Fill => {
                                warnln!("Could not resolve the 'fill' IRI reference '{}'. \
                                         Fallback to 'none'.",
                                         d.iri);
                                d.node.set_attribute(AttributeId::Fill, ValueId::None);
                            }
                            _ => {
                                warnln!("Could not resolve IRI reference: {}.", d.iri);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn skip_current_element(p: &mut svg::Tokenizer) -> Result<(), Error> {
    let mut local_depth = 0;

    loop {
        match p.parse_next()? {
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
            svg::Token::EndOfStream => break,
            _ => {}
        }
    }

    Ok(())
}
