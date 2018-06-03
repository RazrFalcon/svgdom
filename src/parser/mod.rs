// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str;
use std::collections::HashMap;

pub use self::options::*;

mod css;
mod options;
mod text;

use svgtypes::{
    Paint,
    PaintFallback,
    StreamExt,
    StyleParser,
    StyleToken,
};

use svgtypes::xmlparser::{
    self,
    Reference,
    Stream,
    StrSpan,
};

use super::*;

type Result<T> = ::std::result::Result<T, ParserError>;

pub struct NodeSpanData<'a> {
    pub node: Node,
    pub span: StrSpan<'a>,
}

pub struct LinkData<'a> {
    prefix: &'a str,
    attr_id: AttributeId,
    iri: &'a str,
    fallback: Option<PaintFallback>,
    node: Node,
}

pub struct Links<'a> {
    /// List of all parsed IRI and FuncIRI.
    pub list: Vec<LinkData<'a>>,
    /// Store all nodes with id's.
    ///
    /// For performance reasons only.
    pub elems_with_id: HashMap<&'a str, Node>,
}

impl<'a> Links<'a> {
    fn append(
        &mut self,
        prefix: &'a str,
        id: AttributeId,
        iri: &'a str,
        fallback: Option<PaintFallback>,
        node: &Node,
    ) {
        self.list.push(LinkData {
            prefix,
            attr_id: id,
            iri,
            fallback,
            node: node.clone(),
        });
    }
}

pub type Entities<'a> = HashMap<&'a str, StrSpan<'a>>;

pub struct PostData<'a> {
    pub css_list: Vec<StrSpan<'a>>,
    pub links: Links<'a>,
    pub entities: Entities<'a>,
    // List of element with 'class' attribute.
    // We can't process it inplace, because styles can be set after usage.
    pub class_attrs: Vec<NodeSpanData<'a>>,
    // List of style attributes.
    pub style_attrs: Vec<NodeSpanData<'a>>,
}

pub fn parse_svg(text: &str, opt: &ParseOptions) -> Result<Document> {
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
        entities: HashMap::new(),
        class_attrs: Vec::new(),
        style_attrs: Vec::new(),
    };

    let mut doc = Document::new();
    let root = doc.root();

    // Process SVG tokens.
    let mut tokens = xmlparser::Tokenizer::from(text);
    let mut node = root.clone();
    let mut parent = root.clone();
    while let Some(token) = tokens.next() {
        process_token(token?, opt, &mut doc, &mut node, &mut parent, &mut post_data)?;
    }

    // Document must contain any nodes.
    if !root.has_children() {
        return Err(ParserError::EmptyDocument);
    }

    // First element must be an 'svg' element.
    if doc.svg_element().is_none() {
        return Err(ParserError::NoSvgElement);
    }

    // Remove 'style' elements, because their content (CSS)
    // is stored separately and will be processed later.
    doc.drain(root.clone(), |n| n.is_tag_name(ElementId::Style));

    resolve_entity_references(&mut doc, &mut post_data, opt)?;

    if let Err(e) = css::resolve_css(&doc, &mut post_data, opt) {
        if opt.skip_invalid_css {
            warn!("{}.", e);
        } else {
            return Err(e.into());
        }
    }

    // Resolve styles.
    for d in &mut post_data.style_attrs {
        parse_style_attribute(d.span, opt, &post_data.entities,
                              &mut d.node, &mut post_data.links)?;
    }

    resolve_links(&mut post_data.links);

    text::prepare_text(&mut doc);

    Ok(doc)
}

fn process_token<'a>(
    token: xmlparser::Token<'a>,
    opt: &ParseOptions,
    doc: &mut Document,
    node: &mut Node,
    parent: &mut Node,
    post_data: &mut PostData<'a>,
) -> Result<()> {
    macro_rules! create_node {
        ($nodetype:expr, $buf:expr) => {{
            let e = doc.create_node($nodetype, $buf);
            *node = e.clone();
            parent.append(e.clone());
            e
        }}
    }

    match token {
        xmlparser::Token::ElementStart(prefix, local) => {
            let curr_node = match ElementId::from_str(local.to_str()) {
                Some(eid) => {
                    doc.create_element((prefix.to_str(), eid))
                }
                None => {
                    doc.create_element((prefix.to_str(), local.to_str()))
                }
            };

            *node = curr_node.clone();
            parent.append(curr_node);
        }
        xmlparser::Token::Attribute((prefix, local), value) => {
            let prefix = prefix.to_str();
            let local = local.to_str();

            match AttributeId::from_str(local) {
                Some(aid) => {
                    if node.is_svg_element() {
                        parse_svg_attribute(prefix, aid, value, opt, node, post_data)?;
                    } else {
                        node.set_attribute(((prefix, aid.as_str()), value.to_str()));
                    }
                }
                None => {
                    if node.is_svg_element() {
                        parse_non_svg_attribute(prefix, local, value, post_data, node);
                    } else {
                        node.set_attribute(((prefix, local), value.to_str()));
                    }
                }
            }
        }
        xmlparser::Token::ElementEnd(end) => {
            match end {
                xmlparser::ElementEnd::Empty => {}
                xmlparser::ElementEnd::Close(prefix, local) => {
                    let prefix = prefix.to_str();
                    let local = local.to_str();

                    let is_ok = match ElementId::from_str(local) {
                        Some(id) => parent.is_tag_name((prefix, id)),
                        None => parent.is_tag_name((prefix, local)),
                    };

                    if !is_ok {
                        // TODO: simplify
                        let name1 = TagName::from(TagNameRef::from((prefix, local)));
                        return Err(ParserError::UnexpectedCloseTag(node.tag_name().to_string(),
                                                                   name1.to_string()));
                    }

                    if *parent != doc.root() {
                        *parent = parent.parent().unwrap();
                    }
                }
                xmlparser::ElementEnd::Open => {
                    *parent = node.clone();
                }
            }
        }
        xmlparser::Token::Text(s) => {
            if is_inside_style_elem(parent) {
                post_data.css_list.push(s);
            } else {
                create_node!(NodeType::Text, s.to_str());
            }
        }
        xmlparser::Token::Whitespaces(s) => {
            // Whitespaces inside text elements are important.
            if let Some(id) = parent.tag_id() {
                match id {
                      ElementId::Text
                    | ElementId::Tspan
                    | ElementId::Tref => { create_node!(NodeType::Text, s.to_str()); }
                    _ => {}
                }
            }
        }
        xmlparser::Token::Comment(s) => {
            create_node!(NodeType::Comment, s.to_str());
        }
        xmlparser::Token::Cdata(s) => {
            if is_inside_style_elem(parent) {
                post_data.css_list.push(s);
            } else {
                create_node!(NodeType::Cdata, s.to_str());
            }
        }
        xmlparser::Token::Declaration(version, encoding, sa) => {
            let mut n = create_node!(NodeType::Declaration, String::new());
            n.set_attribute((AttributeId::Version, version.to_str()));

            if let Some(encoding) = encoding {
                n.set_attribute((AttributeId::Encoding, encoding.to_str()));
            }

            if let Some(sa) = sa {
                n.set_attribute((AttributeId::Standalone, sa.to_str()));
            }
        }
          xmlparser::Token::DtdStart(_, _)
        | xmlparser::Token::EmptyDtd(_, _)
        | xmlparser::Token::DtdEnd => {
            // Do nothing.
        }
        xmlparser::Token::EntityDeclaration(name, value) => {
            if let xmlparser::EntityDefinition::EntityValue(value) = value {
                post_data.entities.insert(name.to_str(), value);
            }
        }
        xmlparser::Token::ProcessingInstruction(_, _) => {
            // Do nothing.
        }
    }

    // Check that the first element of the doc is 'svg'.
    //
    // Check only when we parsing the root nodes, which is faster.
    if parent.is_root() {
        if let Some((id, _)) = doc.root().children().svg().nth(0) {
            if id != ElementId::Svg {
                return Err(ParserError::NoSvgElement);
            }
        }
    }

    Ok(())
}

fn parse_svg_attribute<'a>(
    prefix: &'a str,
    id: AttributeId,
    value: StrSpan<'a>,
    opt: &ParseOptions,
    node: &mut Node,
    post_data: &mut PostData<'a>,
) -> Result<()> {
    match id {
        AttributeId::Id => {
            node.set_id(value.to_str());
            post_data.links.elems_with_id.insert(value.to_str(), node.clone());
        }
        AttributeId::Style => {
            // We store 'style' attributes for later use.
            post_data.style_attrs.push(NodeSpanData {
                node: node.clone(),
                span: value,
            });
        }
        AttributeId::Class => {
            // TODO: to svgtypes

            // We store 'class' attributes for later use.

            let mut s = Stream::from(value);
            while !s.at_end() {
                s.skip_spaces();

                let class = s.consume_bytes(|s2, _| !s2.starts_with_space());

                post_data.class_attrs.push(NodeSpanData {
                    node: node.clone(),
                    span: class,
                });

                s.skip_spaces();
            }
        }
        _ => {
            parse_svg_attribute_value(prefix, id, value, opt, node, &mut post_data.links,
                                      &post_data.entities)?;
        }
    }

    Ok(())
}

pub fn parse_svg_attribute_value<'a>(
    prefix: &'a str,
    id: AttributeId,
    value: StrSpan<'a>,
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links<'a>,
    entities: &Entities<'a>,
) -> Result<()> {
    let av = _parse_svg_attribute_value(prefix, id, value, opt, entities, node, links);

    match av {
        Ok(av) => {
            if let Some(av) = av {
                match av {
                    AttributeValue::NumberList(ref list) if list.is_empty() => {}
                    AttributeValue::LengthList(ref list) if list.is_empty() => {}
                    _ => node.set_attribute(((prefix, id), av)),
                }
            }
        }
        Err(e) => {
            if opt.skip_invalid_attributes {
                warn!("Attribute '{}' has an invalid value: '{}'.", id, value.to_str());
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

#[inline]
fn f64_bound(min: f64, val: f64, max: f64) -> f64 {
    if val > max {
        return max;
    } else if val < min {
        return min;
    }

    val
}

pub fn _parse_svg_attribute_value<'a>(
    prefix: &'a str,
    aid: AttributeId,
    value: StrSpan<'a>,
    opt: &ParseOptions,
    entities: &Entities<'a>,
    node: &mut Node,
    links: &mut Links<'a>,
) -> Result<Option<AttributeValue>> {
    use AttributeId as AId;

    let eid = node.tag_id().unwrap();

    // 'unicode' attribute can contain spaces.
    let value = if aid != AId::Unicode { value.trim() } else { value };

    {
        let mut stream = Stream::from(value);
        if stream.is_curr_byte_eq(b'&') {
            // TODO: attribute can contain many refs, not only one
            // TODO: advance to the end of the stream
            let r = stream.consume_reference();
            if let Ok(Reference::EntityRef(link)) = r {
                match entities.get(link.to_str()) {
                    Some(link_value) => {
                        parse_svg_attribute_value(prefix, aid, *link_value, opt,
                                                  node, links, entities)?;
                        return Ok(None);
                    }
                    None => {
                        if link.as_bytes()[0] != b'#' {
                            // If link starts with # - than it's probably a Unicode code point.
                            // Otherwise - unknown reference.
                            warn!("Unresolved ENTITY reference: '{}'.", value.to_str());
                        }

                        return Ok(Some(AttributeValue::String(value.to_str().to_string())));
                    }
                }
            }
        }
    }

    if aid == AId::Href && prefix == "xlink" {
        let mut s = Stream::from(value);

        match s.parse_iri() {
            Ok(link) => {
                // Collect links for later processing.
                links.append(prefix, aid, link, None, node);
                return Ok(None);
            }
            Err(_) => {
                return Ok(Some(AttributeValue::String(value.to_str().to_string())));
            }
        }
    }

    if !prefix.is_empty() {
        return Ok(Some(AttributeValue::String(value.to_str().to_string())));
    }

    let av = match aid {
          AId::X  | AId::Y
        | AId::Dx | AId::Dy => {
            // Some attributes can contain different data based on the element type.
            match eid {
                  ElementId::AltGlyph
                | ElementId::Text
                | ElementId::Tref
                | ElementId::Tspan => {
                    AttributeValue::LengthList(LengthList::from_span(value)?)
                }
                _ => {
                    AttributeValue::Length(Length::from_span(value)?)
                }
            }
        }

          AId::X1 | AId::Y1
        | AId::X2 | AId::Y2
        | AId::R
        | AId::Rx | AId::Ry
        | AId::Cx | AId::Cy
        | AId::Fx | AId::Fy
        | AId::Offset
        | AId::Width | AId::Height => {
              AttributeValue::Length(Length::from_span(value)?)
        }

          AId::StrokeDashoffset
        | AId::StrokeMiterlimit
        | AId::StrokeWidth => {
              match value.to_str() {
                  "inherit" => AttributeValue::Inherit,
                  _ => Length::from_span(value)?.into(),
              }
        }

          AId::Opacity
        | AId::FillOpacity
        | AId::FloodOpacity
        | AId::StrokeOpacity
        | AId::StopOpacity => {
            match value.to_str() {
                "inherit" => AttributeValue::Inherit,
                _ => {
                    let mut s = Stream::from(value);
                    let mut n = s.parse_number()?;
                    n = f64_bound(0.0, n, 1.0);
                    AttributeValue::Number(n)
                }
            }
        }

        AId::StrokeDasharray => {
            match value.to_str() {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::LengthList(LengthList::from_span(value)?),
            }
        }

        AId::Fill => {
            // 'fill' in animate-based elements it's another 'fill'
            // https://www.w3.org/TR/SVG/animate.html#FillAttribute
            match eid {
                  ElementId::Set
                | ElementId::Animate
                | ElementId::AnimateColor
                | ElementId::AnimateMotion
                | ElementId::AnimateTransform
                => AttributeValue::String(value.to_str().to_string()),
                _ => {
                    match Paint::from_span(value)? {
                        Paint::None => AttributeValue::None,
                        Paint::Inherit => AttributeValue::Inherit,
                        Paint::CurrentColor => AttributeValue::CurrentColor,
                        Paint::Color(color) => AttributeValue::Color(color),
                        Paint::FuncIRI(link, fallback) => {
                            // collect links for later processing
                            links.append(prefix, aid, link, fallback, node);
                            return Ok(None);
                        }
                    }
                }
            }
        }

        AId::Stroke => {
            match Paint::from_span(value)? {
                Paint::None => AttributeValue::None,
                Paint::Inherit => AttributeValue::Inherit,
                Paint::CurrentColor => AttributeValue::CurrentColor,
                Paint::Color(color) => AttributeValue::Color(color),
                Paint::FuncIRI(link, fallback) => {
                    // Collect links for later processing.
                    links.append(prefix, aid, link, fallback, node);
                    return Ok(None);
                }
            }
        }

          AId::ClipPath
        | AId::Filter
        | AId::Marker
        | AId::MarkerEnd
        | AId::MarkerMid
        | AId::MarkerStart
        | AId::Mask => {
            match value.to_str() {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => {
                    let mut s = Stream::from(value);
                    let link = s.parse_func_iri()?;
                    // collect links for later processing
                    links.append(prefix, aid, link, None, node);
                    return Ok(None);
                }
            }
        }

        AId::Color => {
            match value.to_str() {
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::Color(Color::from_span(value)?),
            }
        }

          AId::LightingColor
        | AId::FloodColor
        | AId::StopColor => {
              match value.to_str() {
                  "inherit" => AttributeValue::Inherit,
                  "currentColor" => AttributeValue::CurrentColor,
                  _ => AttributeValue::Color(Color::from_span(value)?),
              }
        }

          AId::StdDeviation
        | AId::BaseFrequency
        | AId::Rotate => {
            // TODO: 'stdDeviation' can contain only one or two numbers
            AttributeValue::NumberList(NumberList::from_span(value)?)
        }

        AId::Points => {
            AttributeValue::Points(Points::from_span(value)?)
        }

        AId::D => {
            AttributeValue::Path(Path::from_span(value)?)
        }

          AId::Transform
        | AId::GradientTransform
        | AId::PatternTransform => {
            let ts = Transform::from_span(value)?;
            if !ts.is_default() {
                AttributeValue::Transform(Transform::from_span(value)?)
            } else {
                return Ok(None);
            }
        }

        AId::FontSize => {
            let mut s = Stream::from(value);
            match s.parse_length() {
                Ok(l) => AttributeValue::Length(l),
                Err(_) => {
                    if value.to_str() == "inherit" {
                        AttributeValue::Inherit
                    } else {
                        AttributeValue::String(value.to_str().to_string())
                    }
                }
            }
        }

        AId::FontSizeAdjust => {
            match value.to_str() {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => {
                    let mut s = Stream::from(value);
                    AttributeValue::Number(s.parse_number()?)
                }
            }
        }

          AId::Display
        | AId::PointerEvents
        | AId::TextDecoration => {
            match value.to_str() {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::String(value.to_str().to_string()),
            }
        }

          AId::BaselineShift
        | AId::ClipRule
        | AId::ColorInterpolation
        | AId::ColorInterpolationFilters
        | AId::ColorProfile
        | AId::ColorRendering
        | AId::Direction
        | AId::DominantBaseline
        | AId::EnableBackground
        | AId::FillRule
        | AId::FontFamily
        | AId::FontStretch
        | AId::FontStyle
        | AId::FontVariant
        | AId::FontWeight
        | AId::GlyphOrientationVertical
        | AId::ImageRendering
        | AId::Kerning
        | AId::LetterSpacing
        | AId::Overflow
        | AId::ShapeRendering
        | AId::StrokeLinecap
        | AId::StrokeLinejoin
        | AId::TextAnchor
        | AId::TextRendering
        | AId::UnicodeBidi
        | AId::Visibility
        | AId::WordSpacing
        | AId::WritingMode => {
              match value.to_str() {
                  "inherit" => AttributeValue::Inherit,
                  _ => AttributeValue::String(value.to_str().to_string()),
              }
        }

        AId::ViewBox => {
            AttributeValue::ViewBox(ViewBox::from_span(value)?)
        }

        AId::PreserveAspectRatio => {
            AttributeValue::AspectRatio(AspectRatio::from_span(value)?)
        }

        _ => {
            AttributeValue::String(value.to_str().to_string())
        }
    };

    Ok(Some(av))
}

fn parse_non_svg_attribute<'a>(
    prefix: &str,
    name: &str,
    value: StrSpan<'a>,
    post_data: &PostData<'a>,
    node: &mut Node,
) {
    let mut stream = Stream::from(value);
    let new_value = if stream.is_curr_byte_eq(b'&') {
        if let Ok(xmlparser::Reference::EntityRef(link)) = stream.consume_reference() {
            match post_data.entities.get(link.to_str()) {
                Some(link_value) => Some(*link_value),
                None => {
                    warn!("Could not resolve ENTITY: '{}'.", link);
                    None
                }
            }
        } else {
            None
        }
    } else {
        Some(stream.span())
    };

    if let Some(val) = new_value {
        node.set_attribute(((prefix, name), val.to_str()));
    }
}

fn parse_style_attribute<'a>(
    text: StrSpan<'a>,
    opt: &ParseOptions,
    entities: &Entities<'a>,
    node: &mut Node,
    links: &mut Links<'a>,
) -> Result<()> {
    for token in StyleParser::from(text) {
        match token? {
            StyleToken::Attribute(name, value) => {
                match AttributeId::from_str(name.to_str()) {
                    Some(aid) => {
                        parse_svg_attribute_value("", aid, value, opt, node, links, entities)?;
                    }
                    None => {
                        node.set_attribute((name.to_str(), value.to_str()));
                    }
                }
            }
            StyleToken::EntityRef(name) => {
                if let Some(value) = entities.get(name) {
                    parse_style_attribute(*value, opt, entities, node, links)?;
                }
            }
        }
    }

    Ok(())
}

fn resolve_links(links: &mut Links) {
    for d in &mut links.list {
        let name = (d.prefix, d.attr_id);
        match links.elems_with_id.get(d.iri) {
            Some(node) => {
                let res = if d.attr_id == AttributeId::Fill || d.attr_id == AttributeId::Stroke {
                    d.node.set_attribute_checked((name, (node.clone(), d.fallback)))
                } else {
                    d.node.set_attribute_checked((name, node.clone()))
                };

                match res {
                    Ok(_) => {}
                    Err(Error::ElementMustHaveAnId) => {
                        let attr = Attribute::from((name, node.clone()));
                        warn!("Element without an ID cannot be linked. \
                               Attribute {} ignored.", attr);
                    }
                    Err(Error::ElementCrosslink) => {
                        let attr = Attribute::from((name, node.clone()));
                        warn!("Crosslink detected. Attribute {} ignored.", attr);
                    }
                }
            }
            None => {
                let av = match d.fallback {
                    Some(PaintFallback::None) => AttributeValue::None,
                    Some(PaintFallback::CurrentColor) => AttributeValue::CurrentColor,
                    Some(PaintFallback::Color(c)) => AttributeValue::Color(c),
                    None => {
                        if d.attr_id == AttributeId::Fill {
                            warn!("Could not resolve the 'fill' IRI reference: {}. \
                                   Fallback to 'none'.", d.iri);
                            AttributeValue::None
                        } else if d.attr_id == AttributeId::Href {
                            warn!("Could not resolve IRI reference: {}.", d.iri);
                            AttributeValue::String(format!("#{}", d.iri))
                        } else {
                            warn!("Could not resolve FuncIRI reference: {}.", d.iri);
                            AttributeValue::String(format!("url(#{})", d.iri))
                        }
                    }
                };

                d.node.set_attribute((name, av));
            }
        }
    }
}

fn is_inside_style_elem(node: &Node) -> bool {
    if node.is_tag_name(ElementId::Style) {
        let attrs = node.attributes();
        let av = attrs.get_value(AttributeId::Type);
        if let Some(&AttributeValue::String(ref t)) = av {
            if t == "text/css" {
                return true;
            }
        }
    }

    false
}

fn resolve_entity_references<'a>(
    doc: &mut Document,
    post_data: &mut PostData<'a>,
    opt: &ParseOptions,
) -> Result<()> {
    let mut entities = Vec::new();

    for text_node in doc.root().descendants() {
        if !text_node.is_text() {
            continue;
        }

        let text_parent = text_node.parent().unwrap();
        if !text_parent.is_container() {
            continue;
        }

        let text = text_node.text();
        let mut s = Stream::from(text.as_str());
        while !s.at_end() {
            s.skip_spaces();
            if s.get_curr_byte() != Some(b'&') {
                break;
            }

            let ref_name = match s.consume_reference() {
                Ok(Reference::EntityRef(ref name)) => name.to_str(),
                _ => break,
            };

            if let Some(entity) = post_data.entities.get(ref_name).cloned() {
                entities.push((text_node.clone(), entity));
            }

            s.skip_spaces();
            s.skip_bytes(|_, c| c != b'&');
            s.skip_spaces();
        }
    }

    for &(ref text_node, ref entity) in &entities {
        let mut parent = text_node.parent().unwrap();
        let mut node = parent.clone();
        let mut tokens = xmlparser::Tokenizer::from(*entity);
        tokens.set_fragment_mode();
        while let Some(token) = tokens.next() {
            process_token(token?, opt, doc, &mut node, &mut parent, post_data)?;
        }
    }

    for (text_node, _) in entities {
        if !text_node.is_detached() {
            doc.remove_node(text_node.clone());
        }
    }

    Ok(())
}
