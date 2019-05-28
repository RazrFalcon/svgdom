// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::{self, FromStr};

use log::warn;

pub use self::options::*;

use roxmltree::{
    self,
    TextPos,
};

use svgtypes::{
    Paint,
    PaintFallback,
    PathParser,
    Stream,
    StyleParser,
};

use super::*;

mod css;
mod options;
mod text;

pub struct NodeStringData {
    pub node: Node,
    pub text: String,
    pub value_pos: usize,
}

pub struct LinkData {
    attr_id: AttributeId,
    iri: String,
    fallback: Option<PaintFallback>,
    node: Node,
}

pub struct Links {
    /// List of all parsed IRI and FuncIRI.
    pub list: Vec<LinkData>,
}

impl Links {
    fn append(
        &mut self,
        id: AttributeId,
        iri: &str,
        fallback: Option<PaintFallback>,
        node: &Node,
    ) {
        self.list.push(LinkData {
            attr_id: id,
            iri: iri.to_string(),
            fallback,
            node: node.clone(),
        });
    }
}

pub struct PostData {
    pub links: Links,
    // List of element with 'class' attribute.
    // We can't process it inplace, because styles can be set after usage.
    pub class_attrs: Vec<NodeStringData>,
    // List of style attributes.
    pub style_attrs: Vec<NodeStringData>,
}

pub fn parse_svg(text: &str, opt: &ParseOptions) -> Result<Document, ParserError> {
    let ro_doc = roxmltree::Document::parse(text)?;

    // Since we not only parsing, but also converting an SVG structure,
    // we can't do everything in one take.
    // At first, we create nodes structure with attributes.
    // Than apply CSS. And then ungroup style attributes.
    // Order is important, otherwise we get rendering error.
    let mut post_data = PostData {
        links: Links {
            list: Vec::new(),
        },
        class_attrs: Vec::new(),
        style_attrs: Vec::new(),
    };

    let mut doc = Document::new();
    let root = doc.root();
    let mut parent = root.clone();

    for child in ro_doc.root().children() {
        process_node(&ro_doc, child, opt, &mut post_data, &mut doc, &mut parent)?;
    }

    // First element must be an 'svg' element.
    if doc.svg_element().is_none() {
        return Err(ParserError::NoSvgElement);
    }

    // Remove 'style' elements, because their content (CSS)
    // is stored separately and will be processed later.
    doc.drain(root.clone(), |n| n.is_tag_name(ElementId::Style));

    css::resolve_css(&ro_doc, &doc, &mut post_data, opt)?;

    // Resolve styles.
    for d in &mut post_data.style_attrs {
        parse_style_attribute(&ro_doc, &d.text, d.value_pos, opt,
                              &mut d.node, &mut post_data.links)?;
    }

    resolve_links(&doc, &mut post_data.links);

    text::prepare_text(&mut doc);

    Ok(doc)
}

fn process_node(
    ro_doc: &roxmltree::Document,
    xml_node: roxmltree::Node,
    opt: &ParseOptions,
    post_data: &mut PostData,
    doc: &mut Document,
    parent: &mut Node,
) -> Result<(), ParserError> {
    match xml_node.node_type() {
        roxmltree::NodeType::Element => {
            if xml_node.tag_name().namespace() != Some("http://www.w3.org/2000/svg") {
                return Ok(());
            }

            let tag_name = xml_node.tag_name();
            let local = tag_name.name();
            let mut e = match ElementId::from_str(local) {
                Some(eid) => {
                    doc.create_element(eid)
                }
                None => {
                    return Ok(());
                }
            };

            for attr in xml_node.attributes() {
                match attr.namespace() {
                    None |
                    Some("http://www.w3.org/2000/svg") |
                    Some("http://www.w3.org/1999/xlink") |
                    Some("http://www.w3.org/XML/1998/namespace") => {}
                    _ => continue,
                }

                if let Some(aid) = AttributeId::from_str(attr.name()) {
                    if e.is_svg_element() {
                        parse_svg_attribute(ro_doc, aid, attr.value(), attr.value_range().start,
                                            opt, &mut e, post_data)?;
                    }
                }
            }

            parent.append(e.clone());

            if xml_node.is_element() && xml_node.has_children() {
                for child in xml_node.children() {
                    process_node(ro_doc, child, opt, post_data, doc, &mut e)?;
                }
            }
        }
        roxmltree::NodeType::Text => {
            let text = xml_node.text().unwrap();
            if text.trim().is_empty() {
                // Whitespaces inside text elements are important.
                if let Some(id) = parent.tag_id() {
                    match id {
                          ElementId::Text
                        | ElementId::Tspan
                        | ElementId::Tref => {
                            let n = doc.create_node(NodeType::Text, text);
                            parent.append(n);
                        }
                        _ => {}
                    }
                }
            } else {
                let n = doc.create_node(NodeType::Text, xml_node.text().unwrap());
                parent.append(n);
            }
        }
        roxmltree::NodeType::Comment => {
            let n = doc.create_node(NodeType::Comment, xml_node.text().unwrap());
            parent.append(n);
        }
        _ => {}
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
    ro_doc: &roxmltree::Document,
    id: AttributeId,
    value: &'a str,
    value_pos: usize,
    opt: &ParseOptions,
    node: &mut Node,
    post_data: &mut PostData,
) -> Result<(), ParserError> {
    match id {
        AttributeId::Id => {
            node.set_id(value);
        }
        AttributeId::Style => {
            // We store 'style' attributes for later use.
            post_data.style_attrs.push(NodeStringData {
                node: node.clone(),
                text: value.to_string(),
                value_pos,
            });
        }
        AttributeId::Class => {
            // TODO: to svgtypes

            // We store 'class' attributes for later use.

            let mut s = Stream::from(value);
            while !s.at_end() {
                s.skip_spaces();

                let class = s.consume_bytes(|s2, _| !s2.starts_with_space());

                post_data.class_attrs.push(NodeStringData {
                    node: node.clone(),
                    text: class.to_string(),
                    value_pos,
                });

                s.skip_spaces();
            }
        }
        _ => {
            parse_svg_attribute_value(ro_doc, id, value, value_pos, opt,
                                      node, &mut post_data.links)?;
        }
    }

    Ok(())
}

pub fn parse_svg_attribute_value<'a>(
    ro_doc: &roxmltree::Document,
    id: AttributeId,
    value: &'a str,
    value_pos: usize,
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links,
) -> Result<(), ParserError> {
    let av = _parse_svg_attribute_value(ro_doc, id, value, value_pos, node, links);

    match av {
        Ok(av) => {
            if let Some(av) = av {
                match av {
                    AttributeValue::NumberList(ref list) if list.is_empty() => {}
                    AttributeValue::LengthList(ref list) if list.is_empty() => {}
                    AttributeValue::Path(ref path) if path.is_empty() => {}
                    _ => node.set_attribute((id, av)),
                }
            }
        }
        Err(_) => {
            if opt.skip_invalid_attributes {
                warn!("Attribute '{}' has an invalid value: '{}'.", id, value);
            } else {
                let pos = ro_doc.text_pos_at(value_pos);
                return Err(ParserError::InvalidAttributeValue(pos));
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
    ro_doc: &roxmltree::Document,
    aid: AttributeId,
    value: &'a str,
    value_pos: usize,
    node: &mut Node,
    links: &mut Links,
) -> Result<Option<AttributeValue>, svgtypes::Error> {
    use crate::AttributeId as AId;

    let eid = node.tag_id().unwrap();

    // 'unicode' attribute can contain spaces.
    let value = if aid != AId::Unicode { value.trim() } else { value };

    let av = match aid {
        AId::Href => {
            match Stream::from(value).parse_iri() {
                Ok(link) => {
                    // Collect links for later processing.
                    links.append(aid, link, None, node);
                    return Ok(None);
                }
                Err(_) => {
                    return Ok(Some(AttributeValue::String(value.to_string())));
                }
            }
        }

          AId::X  | AId::Y
        | AId::Dx | AId::Dy => {
            // Some attributes can contain different data based on the element type.
            match eid {
                  ElementId::AltGlyph
                | ElementId::Text
                | ElementId::Tref
                | ElementId::Tspan => {
                    AttributeValue::LengthList(LengthList::from_str(value)?)
                }
                _ => {
                    AttributeValue::Length(Length::from_str(value)?)
                }
            }
        }

          AId::X1 | AId::Y1
        | AId::X2 | AId::Y2
        | AId::R
        | AId::Rx | AId::Ry
        | AId::Cx | AId::Cy
        | AId::Fx | AId::Fy
        | AId::RefX | AId::RefY
        | AId::Width | AId::Height
        | AId::MarkerWidth | AId::MarkerHeight
        | AId::StartOffset => {
            AttributeValue::Length(Length::from_str(value)?)
        }

        AId::Offset => {
            // offset = <number> | <percentage>
            let l = Length::from_str(value)?;
            if l.unit == LengthUnit::None || l.unit == LengthUnit::Percent {
                AttributeValue::Length(l)
            } else {
                return Err(svgtypes::Error::InvalidValue);
            }
        }

          AId::StrokeDashoffset
        | AId::StrokeWidth => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => Length::from_str(value)?.into(),
            }
        }

        AId::StrokeMiterlimit => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => parse_number(value)?.into(),
            }
        }

          AId::Opacity
        | AId::FillOpacity
        | AId::FloodOpacity
        | AId::StrokeOpacity
        | AId::StopOpacity => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => {
                    let n = parse_number(value)?;
                    let n = f64_bound(0.0, n, 1.0);
                    AttributeValue::Number(n)
                }
            }
        }

          AId::K1
        | AId::K2
        | AId::K3
        | AId::K4 => {
            let n = parse_number(value)?;
            let n = f64_bound(0.0, n, 1.0);
            AttributeValue::Number(n)
        }

        AId::StrokeDasharray => {
            match value {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::LengthList(LengthList::from_str(value)?),
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
                => AttributeValue::String(value.to_string()),
                _ => {
                    match Paint::from_str(value)? {
                        Paint::None => AttributeValue::None,
                        Paint::Inherit => AttributeValue::Inherit,
                        Paint::CurrentColor => AttributeValue::CurrentColor,
                        Paint::Color(color) => AttributeValue::Color(color),
                        Paint::FuncIRI(link, fallback) => {
                            // Collect links for later processing.
                            links.append(aid, link, fallback, node);
                            return Ok(None);
                        }
                    }
                }
            }
        }

        AId::Stroke => {
            match Paint::from_str(value)? {
                Paint::None => AttributeValue::None,
                Paint::Inherit => AttributeValue::Inherit,
                Paint::CurrentColor => AttributeValue::CurrentColor,
                Paint::Color(color) => AttributeValue::Color(color),
                Paint::FuncIRI(link, fallback) => {
                    // Collect links for later processing.
                    links.append(aid, link, fallback, node);
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
            match value {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => {
                    let mut s = Stream::from(value);
                    let link = s.parse_func_iri()?;
                    // collect links for later processing
                    links.append(aid, link, None, node);
                    return Ok(None);
                }
            }
        }

        AId::Color => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::Color(Color::from_str(value)?),
            }
        }

          AId::LightingColor
        | AId::FloodColor
        | AId::StopColor => {
            match value {
                "inherit" => AttributeValue::Inherit,
                "currentColor" => AttributeValue::CurrentColor,
                _ => AttributeValue::Color(Color::from_str(value)?),
            }
        }

          AId::StdDeviation
        | AId::BaseFrequency
        | AId::Rotate => {
            // TODO: 'stdDeviation' can contain only one or two numbers
            AttributeValue::NumberList(NumberList::from_str(value)?)
        }

        AId::Points => {
            AttributeValue::Points(Points::from_str(value)?)
        }

        AId::D => {
            let mut data = Vec::new();
            for token in PathParser::from(value) {
                match token {
                    Ok(token) => data.push(token),
                    Err(_) => {
                        // By the SVG spec, any invalid data inside the path data
                        // should stop parsing of this path, but not the whole document.
                        let pos = ro_doc.text_pos_at(value_pos);
                        warn!("A path attribute at {} was parsed partially \
                               due to an invalid data.", pos);
                        break;
                    }
                }
            }

            AttributeValue::Path(Path(data))
        }

          AId::Transform
        | AId::GradientTransform
        | AId::PatternTransform => {
            let ts = Transform::from_str(value)?;
            if !ts.is_default() {
                AttributeValue::Transform(Transform::from_str(value)?)
            } else {
                return Ok(None);
            }
        }

        AId::FontSize => {
            match Length::from_str(value) {
                Ok(l) => AttributeValue::Length(l),
                Err(_) => {
                    if value == "inherit" {
                        AttributeValue::Inherit
                    } else {
                        AttributeValue::String(value.to_string())
                    }
                }
            }
        }

        AId::FontSizeAdjust => {
            match value {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => parse_number(value)?.into(),
            }
        }

          AId::Display
        | AId::PointerEvents
        | AId::TextDecoration => {
            match value {
                "none" => AttributeValue::None,
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::String(value.to_string()),
            }
        }

          AId::ClipRule
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
        | AId::ImageRendering
        | AId::Kerning
        | AId::Overflow
        | AId::ShapeRendering
        | AId::StrokeLinecap
        | AId::StrokeLinejoin
        | AId::TextAnchor
        | AId::TextRendering
        | AId::UnicodeBidi
        | AId::Visibility
        | AId::WritingMode => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::String(value.to_string()),
            }
        }

          AId::LetterSpacing
        | AId::WordSpacing => {
              match value {
                  "inherit" => AttributeValue::Inherit,
                  "normal" => AttributeValue::String(value.to_string()),
                  _ => AttributeValue::Length(Length::from_str(value)?),
              }
        }

        AId::BaselineShift => {
            match value {
                "inherit" => AttributeValue::Inherit,
                "baseline" | "sub" | "super" => AttributeValue::String(value.to_string()),
                _ => AttributeValue::Length(Length::from_str(value)?),
            }
        }

        AId::Orient => {
            match value {
                "auto" => AttributeValue::String(value.to_string()),
                _ => AttributeValue::Angle(Angle::from_str(value)?),
            }
        }

        AId::GlyphOrientationHorizontal => {
            match value {
                "inherit" => AttributeValue::Inherit,
                _ => AttributeValue::Angle(Angle::from_str(value)?),
            }
        }

        AId::GlyphOrientationVertical => {
            match value {
                "inherit" => AttributeValue::Inherit,
                "auto" => AttributeValue::String(value.to_string()),
                _ => AttributeValue::Angle(Angle::from_str(value)?),
            }
        }

        AId::ViewBox => {
            AttributeValue::ViewBox(ViewBox::from_str(value)?)
        }

        AId::PreserveAspectRatio => {
            AttributeValue::AspectRatio(AspectRatio::from_str(value)?)
        }

        _ => {
            AttributeValue::String(value.to_string())
        }
    };

    Ok(Some(av))
}

fn parse_number(value: &str) -> Result<f64, svgtypes::Error> {
    let mut s = Stream::from(value);
    let n = s.parse_number()?;

    if !s.at_end() {
        return Err(svgtypes::Error::InvalidValue);
    }

    Ok(n)
}

fn parse_style_attribute(
    ro_doc: &roxmltree::Document,
    value: &str,
    value_pos: usize,
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links,
) -> Result<(), ParserError> {
    for token in StyleParser::from(value) {
        let (name, value) = match token {
            Ok(v) => v,
            Err(_) => {
                // TODO: this
                let pos = TextPos::new(0, 0);
                return Err(ParserError::InvalidAttributeValue(pos));
            }
        };

        match AttributeId::from_str(name) {
            Some(aid) => {
                parse_svg_attribute_value(ro_doc, aid, value, value_pos, opt, node, links)?;
            }
            None => {
                node.set_attribute((name, value));
            }
        }
    }

    Ok(())
}

fn resolve_links(doc: &Document, links: &mut Links) {
    for d in &mut links.list {
        match doc.root().descendants().find(|n| *n.id() == d.iri) {
            Some(node) => {
                let res = if d.attr_id == AttributeId::Fill || d.attr_id == AttributeId::Stroke {
                    d.node.set_attribute_checked((d.attr_id, (node.clone(), d.fallback)))
                } else {
                    d.node.set_attribute_checked((d.attr_id, node.clone()))
                };

                match res {
                    Ok(_) => {}
                    Err(Error::ElementMustHaveAnId) => {
                        // TODO: unreachable?
                        let attr = Attribute::from((d.attr_id, node.clone()));
                        warn!("Element without an ID cannot be linked. \
                               Attribute {} ignored.", attr);
                    }
                    Err(Error::ElementCrosslink) => {
                        let attr = Attribute::from((d.attr_id, node.clone()));
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
                            warn!("Could not resolve a 'fill' IRI reference: {}. \
                                   Fallback to 'none'.", d.iri);
                            AttributeValue::None
                        } else if d.attr_id == AttributeId::Href {
                            warn!("Could not resolve an IRI reference: {}.", d.iri);
                            AttributeValue::String(format!("#{}", d.iri))
                        } else {
                            warn!("Could not resolve a FuncIRI reference: {}.", d.iri);
                            AttributeValue::String(format!("url(#{})", d.iri))
                        }
                    }
                };

                d.node.set_attribute((d.attr_id, av));
            }
        }
    }
}
