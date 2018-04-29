// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Write;
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

use error::Result;
use {
    AspectRatio,
    Attribute,
    AttributeId,
    AttributeValue,
    Color,
    Document,
    ElementId,
    Error,
    FilterSvg,
    FromSpan,
    Length,
    LengthList,
    LengthUnit,
    TagName,
    TagNameRef,
    Node,
    NodeType,
    NumberList,
    ParseOptions,
    Path,
    Points,
    Transform,
    ViewBox,
};

type StreamResult<T> = ::std::result::Result<T, Error>;

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
    let mut doc = Document::new();
    let mut root = doc.root();

    let mut tokens = xmlparser::Tokenizer::from(text);

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

    // process SVG tokens
    let mut node: Option<Node> = None;

    while let Some(token) = tokens.next() {
        process_token(&mut doc, token?,
                      &mut node, &mut root,
                      &mut post_data, opt)?
    }

    // document must contain any children
    if !root.has_children() {
        return Err(Error::EmptyDocument);
    }

    // first element must be an 'svg'
    match root.children().svg().nth(0) {
        Some((id, _)) => {
            if id != ElementId::Svg {
                return Err(Error::NoSvgElement);
            }
        }
        None => {
            return Err(Error::NoSvgElement);
        }
    }

    doc.drain(root.clone(), |n| n.is_tag_name(ElementId::Style));

    if !opt.parse_unknown_elements {
        doc.drain(root.clone(), |n|
            n.is_element() && n.tag_id().is_none()
        );
    }

    if let Err(e) = css::resolve_css(&doc, &mut post_data, opt) {
        if opt.skip_invalid_css {
            warn!("{}.", e);
        } else {
            return Err(e.into());
        }
    }

    // resolve styles
    for d in &mut post_data.style_attrs {
        parse_style_attribute(&mut d.node, d.span, &mut post_data.links,
                              &post_data.entities, opt)?;
    }

    resolve_links(&mut post_data.links, opt)?;

    text::prepare_text(&mut doc);

    Ok(doc)
}

fn process_token<'a>(
    doc: &mut Document,
    token: xmlparser::Token<'a>,
    node: &mut Option<Node>,
    parent: &mut Node,
    post_data: &mut PostData<'a>,
    opt: &ParseOptions,
) -> Result<()> {
    macro_rules! create_node {
        ($nodetype:expr, $buf:expr) => ({
            let e = doc.create_node($nodetype, $buf);
            *node = Some(e.clone());
            parent.append(e);
        })
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

            *node = Some(curr_node.clone());
            parent.append(curr_node);
        }
        xmlparser::Token::Attribute((prefix, local), value) => {
            let curr_node = node.as_mut().unwrap();
            match AttributeId::from_str(local.to_str()) {
                Some(aid) => {
                    if curr_node.is_svg_element() {
                        parse_svg_attribute(curr_node, prefix.to_str(), aid, value, post_data, opt)?;
                    } else {
                        curr_node.set_attribute(((prefix.to_str(), aid.as_str()), value.to_str()));
                    }
                }
                None => {
                    if opt.parse_unknown_attributes {
                        if curr_node.is_svg_element() {
                            parse_non_svg_attribute(curr_node, prefix.to_str(), local.to_str(), value, post_data);
                        } else {
                            curr_node.set_attribute(((prefix.to_str(), local.to_str()), value.to_str()));
                        }
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

                    if let Some(ref n) = *node {
                        let is_ok = match ElementId::from_str(local) {
                            Some(id) => parent.is_tag_name((prefix, id)),
                            None => parent.is_tag_name((prefix, local)),
                        };

                        if !is_ok {
                            let name1 = TagName::from(TagNameRef::from((prefix, local)));
                            return Err(Error::UnexpectedCloseTag(n.tag_name().to_string(),
                                                                 name1.to_string()));
                        }
                    } else {
                        unreachable!();
                    }

                    if *parent != doc.root() {
                        *parent = parent.parent().unwrap();
                    }
                }
                xmlparser::ElementEnd::Open => {
                    if let Some(ref n) = *node {
                        *parent = n.clone();
                    }
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
                    | ElementId::Tref => create_node!(NodeType::Text, s.to_str()),
                    _ => {}
                }
            }
        }
        xmlparser::Token::Comment(s) => {
            if opt.parse_comments {
                create_node!(NodeType::Comment, s.to_str())
            }
        }
        xmlparser::Token::Cdata(s) => {
            if is_inside_style_elem(parent) {
                post_data.css_list.push(s);
            } else {
                create_node!(NodeType::Cdata, s.to_str());
            }
        }
        xmlparser::Token::Declaration(version, encoding, sa) => {
            // TODO: check that it UTF-8

            if opt.parse_declarations {
                // TODO: crate a proper way to store this values
                let mut s = format!("version=\"{}\"", version);

                if let Some(encoding) = encoding {
                    write!(&mut s, " encoding=\"{}\"", encoding).unwrap();
                }

                if let Some(sa) = sa {
                    write!(&mut s, " standalone=\"{}\"", sa).unwrap();
                }

                create_node!(NodeType::Declaration, s);
            }
        }
          xmlparser::Token::DtdStart(_, _)
        | xmlparser::Token::EmptyDtd(_, _)
        | xmlparser::Token::DtdEnd => {
            // do nothing
        }
        xmlparser::Token::EntityDeclaration(name, value) => {
            match value {
                xmlparser::EntityDefinition::EntityValue(value) => {
                    // check that ENTITY does not contain an element(s)
                    if value.to_str().trim().starts_with("<") {
                        let s = Stream::from(value);
                        return Err(Error::UnsupportedEntity(s.gen_error_pos()));
                    }

                    post_data.entities.insert(name.to_str(), value);
                }
                _ => {
                    // do nothing
                }
            }
        }
        xmlparser::Token::ProcessingInstruction(_, _) => {
            // do nothing
        }
    }

    // check for 'svg' element only when we parsing root nodes,
    // which is faster
    if parent.is_root() {
        // check that the first element of the doc is 'svg'
        if let Some((id, _)) = doc.root().children().svg().nth(0) {
            if id != ElementId::Svg {
                return Err(Error::NoSvgElement);
            }
        }
    }

    Ok(())
}

fn parse_svg_attribute<'a>(
    node: &mut Node,
    prefix: &'a str,
    id: AttributeId,
    value: StrSpan<'a>,
    post_data: &mut PostData<'a>,
    opt: &ParseOptions,
) -> StreamResult<()> {
    match id {
        AttributeId::Id => {
            node.set_id(value.to_str());
            post_data.links.elems_with_id.insert(value.to_str(), node.clone());
        }
        AttributeId::Style => {
            // we store 'class' attributes for later use
            post_data.style_attrs.push(NodeSpanData {
                node: node.clone(),
                span: value,
            })
        }
        AttributeId::Class => {
            // TODO: to svgtypes

            // we store 'class' attributes for later use

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
            parse_svg_attribute_value(node, prefix, id, value, &mut post_data.links,
                                      &post_data.entities, opt)?;
        }
    }

    Ok(())
}

pub fn parse_svg_attribute_value<'a>(
    node: &mut Node,
    prefix: &'a str,
    id: AttributeId,
    value: StrSpan<'a>,
    links: &mut Links<'a>,
    entities: &Entities<'a>,
    opt: &ParseOptions,
) -> StreamResult<()> {
    let av = _parse_svg_attribute_value(node, prefix, id, value, links, entities, opt);

    match av {
        Ok(av) => {
            if let Some(mut av) = av {
                if let AttributeValue::NumberList(ref list) = av {
                    if list.is_empty() {
                        return Ok(());
                    }
                }

                if let AttributeValue::LengthList(ref mut list) = av {
                    if list.is_empty() {
                        return Ok(());
                    }

                    for len in list.iter_mut() {
                        // replace 'px' with 'none' when 'parse_px_unit' option is disabled
                        if !opt.parse_px_unit && len.unit == LengthUnit::Px {
                            len.unit = LengthUnit::None;
                        }
                    }
                }

                if let AttributeValue::Length(ref mut len) = av {
                    // replace 'px' with 'none' when 'parse_px_unit' option is disabled
                    if !opt.parse_px_unit && len.unit == LengthUnit::Px {
                        len.unit = LengthUnit::None;
                    }
                }

                node.set_attribute(((prefix, id), av));
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
    node: &mut Node,
    prefix: &'a str,
    aid: AttributeId,
    value: StrSpan<'a>,
    links: &mut Links<'a>,
    entities: &Entities<'a>,
    opt: &ParseOptions,
) -> StreamResult<Option<AttributeValue>> {
    use AttributeId as AId;

    let eid = node.tag_id().unwrap();

    // 'unicode' attribute can contain spaces
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
                        parse_svg_attribute_value(node, prefix, aid, *link_value, links, entities, opt)?;
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
                // collect links for later processing
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
            // some attributes can contain different data based on the element type
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
                    // collect links for later processing
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
        | AId::BaseFrequency => {
            // TODO: this attributes can contain only one or two numbers
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
    node: &mut Node,
    prefix: &str,
    name: &str,
    value: StrSpan<'a>,
    post_data: &PostData<'a>,
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
    node: &mut Node,
    span: StrSpan<'a>,
    links: &mut Links<'a>,
    entities: &Entities<'a>,
    opt: &ParseOptions,
) -> Result<()> {
    for token in StyleParser::from(span) {
        match token? {
            StyleToken::Attribute(name, value) => {
                match AttributeId::from_str(name.to_str()) {
                    Some(aid) => {
                        parse_svg_attribute_value(node, "", aid, value, links, entities, opt)?;
                    }
                    None => {
                        if opt.parse_unknown_attributes {
                            node.set_attribute((name.to_str(), value.to_str()));
                        }
                    }
                }
            }
            StyleToken::EntityRef(name) => {
                if let Some(value) = entities.get(name) {
                    parse_style_attribute(node, *value, links, entities, opt)?;
                }
            }
        }
    }

    Ok(())
}

fn resolve_links(links: &mut Links, opt: &ParseOptions) -> Result<()> {
    for mut d in &mut links.list {
        match links.elems_with_id.get(d.iri) {
            Some(node) => {
                // The SVG uses a fallback paint value not only when the FuncIRI is invalid,
                // but also when a referenced element is invalid.
                // And we don't know is it invalid or not.
                // It will take tonnes of code to validate all supported referenced elements,
                // so we just show an error.
                match d.fallback {
                    Some(_) => {
                        if opt.skip_paint_fallback {
                            warn!("Paint fallback is not supported.");
                            d.node.set_attribute_checked(((d.prefix, d.attr_id), node.clone()))?;
                        } else {
                            let s = d.iri.to_string();
                            return Err(Error::UnsupportedPaintFallback(s));
                        }
                    }
                    None => {
                        let res = d.node.set_attribute_checked(((d.prefix, d.attr_id), node.clone()));
                        match res {
                            Ok(_) => {}
                            Err(Error::ElementCrosslink) => {
                                if opt.skip_elements_crosslink {
                                    let attr = Attribute::from(((d.prefix, d.attr_id), node.clone()));
                                    warn!("Crosslink detected. Attribute {} ignored.", attr);
                                } else {
                                    return Err(Error::ElementCrosslink)
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
            None => {
                resolve_fallback(&mut d)?;
            }
        }
    }

    Ok(())
}

fn resolve_fallback(d: &mut LinkData) -> Result<()> {
    // check that <paint> contains a fallback value before showing a warning
    match d.fallback {
        Some(fallback) => {
            match fallback {
                PaintFallback::None => {
                    d.node.set_attribute(((d.prefix, d.attr_id), AttributeValue::None));
                }
                PaintFallback::CurrentColor => {
                    d.node.set_attribute(((d.prefix, d.attr_id), AttributeValue::CurrentColor));
                }
                PaintFallback::Color(c) => {
                    d.node.set_attribute(((d.prefix, d.attr_id), Color::new(c.red, c.green, c.blue)));
                }
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

                    let flag = d.node.ancestors().any(|n| {
                        n.is_tag_name(ElementId::Mask)
                            || n.is_tag_name(ElementId::ClipPath)
                            || n.is_tag_name(ElementId::Marker)
                    });

                    if flag {
                        // If our element is inside one of this elements - then do nothing.
                        // I can't find explanation of this in the SVG spec, but it works.
                        // Probably because this elements only care about a shape,
                        // not a style.
                        warn!("Could not resolve IRI reference: {}.", d.iri);
                    } else {
                        // Imitate invisible element.
                        warn!("Unresolved 'filter' IRI reference: {}. \
                               Marking the element as invisible.",
                              d.iri);
                        d.node.set_attribute((AttributeId::Visibility, "hidden"));
                    }
                }
                AttributeId::Fill => {
                    warn!("Could not resolve the 'fill' IRI reference: {}. \
                           Fallback to 'none'.",
                          d.iri);
                    d.node.set_attribute((AttributeId::Fill, AttributeValue::None));
                }
                _ => {
                    warn!("Could not resolve IRI reference: {}.", d.iri);
                }
            }
        }
    }

    Ok(())
}

fn is_inside_style_elem(node: &Node) -> bool {
    if node.is_tag_name(ElementId::Style) {
        let attrs = node.attributes();
        let av = attrs.get_value(AttributeId::Type);
        if let Some(&AttributeValue::String(ref t)) = av {
            if t != "text/css" {
                return false;
            }
        }

        return true;
    }

    false
}
