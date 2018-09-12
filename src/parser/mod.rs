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

use roxmltree;

use svgtypes::{
    Paint,
    PaintFallback,
    StreamExt,
    StyleParser,
};

use svgtypes::xmlparser::{
    Stream,
    StrSpan,
};

use super::*;

mod css;
mod options;
mod text;

type Result<T> = ::std::result::Result<T, ParserError>;

pub struct NodeStringData {
    pub node: Node,
    pub text: String,
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
    /// Store all nodes with id's.
    ///
    /// For performance reasons only.
    pub elems_with_id: HashMap<String, Node>,
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

pub fn parse_svg(text: &str, opt: &ParseOptions) -> Result<Document> {
    let ro_doc = roxmltree::Document::parse(text)?;

    // Since we not only parsing, but also converting an SVG structure,
    // we can't do everything in one take.
    // At first, we create nodes structure with attributes.
    // Than apply CSS. And then ungroup style attributes.
    // Order is important, otherwise we get rendering error.
    let mut post_data = PostData {
        links: Links {
            list: Vec::new(),
            elems_with_id: HashMap::new(),
        },
        class_attrs: Vec::new(),
        style_attrs: Vec::new(),
    };

    let mut doc = Document::new();
    let root = doc.root();
    let mut parent = root.clone();

    for child in ro_doc.root().children() {
        process_node(child, opt, &mut post_data, &mut doc, &mut parent)?;
    }


    // First element must be an 'svg' element.
    if doc.svg_element().is_none() {
        return Err(ParserError::NoSvgElement);
    }

    // Remove 'style' elements, because their content (CSS)
    // is stored separately and will be processed later.
    doc.drain(root.clone(), |n| n.is_tag_name(ElementId::Style));

    if let Err(e) = css::resolve_css(&ro_doc, &doc, &mut post_data, opt) {
        if opt.skip_invalid_css {
            warn!("{}.", e);
        } else {
            return Err(e.into());
        }
    }

    // Resolve styles.
    for d in &mut post_data.style_attrs {
        parse_style_attribute(&d.text, opt, &mut d.node, &mut post_data.links)?;
    }

    resolve_links(&mut post_data.links);

    text::prepare_text(&mut doc);

    Ok(doc)
}

fn process_node(
    xml_node: roxmltree::Node,
    opt: &ParseOptions,
    post_data: &mut PostData,
    doc: &mut Document,
    parent: &mut Node,
) -> Result<()> {
    match xml_node.node_type() {
        roxmltree::NodeType::Element => {
            if xml_node.tag_name().namespace() != "http://www.w3.org/2000/svg" {
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
                    "" |
                    "http://www.w3.org/2000/svg" |
                    "http://www.w3.org/1999/xlink" |
                    "http://www.w3.org/XML/1998/namespace" => {}
                    _ => continue,
                }

                let local = attr.name();
                let value = StrSpan::from(attr.value());

                if let Some(aid) = AttributeId::from_str(local) {
                    if e.is_svg_element() {
                        parse_svg_attribute(aid, value, opt, &mut e, post_data)?;
                    }
                }
            }

            parent.append(e.clone());

            if xml_node.is_element() && xml_node.has_children() {
                for child in xml_node.children() {
                    process_node(child, opt, post_data, doc, &mut e)?;
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
    id: AttributeId,
    value: StrSpan<'a>,
    opt: &ParseOptions,
    node: &mut Node,
    post_data: &mut PostData,
) -> Result<()> {
    match id {
        AttributeId::Id => {
            node.set_id(value.to_str());
            post_data.links.elems_with_id.insert(value.to_str().to_owned(), node.clone());
        }
        AttributeId::Style => {
            // We store 'style' attributes for later use.
            post_data.style_attrs.push(NodeStringData {
                node: node.clone(),
                text: value.to_string(),
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
                });

                s.skip_spaces();
            }
        }
        _ => {
            parse_svg_attribute_value(id, value, opt, node, &mut post_data.links)?;
        }
    }

    Ok(())
}

pub fn parse_svg_attribute_value<'a>(
    id: AttributeId,
    value: StrSpan<'a>,
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links,
) -> Result<()> {
    let av = _parse_svg_attribute_value(id, value, node, links);

    match av {
        Ok(av) => {
            if let Some(av) = av {
                match av {
                    AttributeValue::NumberList(ref list) if list.is_empty() => {}
                    AttributeValue::LengthList(ref list) if list.is_empty() => {}
                    _ => node.set_attribute((id, av)),
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
    aid: AttributeId,
    value: StrSpan<'a>,
    node: &mut Node,
    links: &mut Links,
) -> Result<Option<AttributeValue>> {
    use AttributeId as AId;

    let eid = node.tag_id().unwrap();

    // 'unicode' attribute can contain spaces.
    let value = if aid != AId::Unicode { value.trim() } else { value };

    if aid == AId::Href {
        let mut s = Stream::from(value);

        match s.parse_iri() {
            Ok(link) => {
                // Collect links for later processing.
                links.append(aid, link, None, node);
                return Ok(None);
            }
            Err(_) => {
                return Ok(Some(AttributeValue::String(value.to_str().to_string())));
            }
        }
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
                            links.append(aid, link, fallback, node);
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
            match value.to_str() {
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

fn parse_style_attribute(
    text: &str,
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links,
) -> Result<()> {
    for token in StyleParser::from(text) {
        let (name, value) = token?;
        match AttributeId::from_str(name.to_str()) {
            Some(aid) => {
                parse_svg_attribute_value(aid, value, opt, node, links)?;
            }
            None => {
                node.set_attribute((name.to_str(), value.to_str()));
            }
        }
    }

    Ok(())
}

fn resolve_links(links: &mut Links) {
    for d in &mut links.list {
        match links.elems_with_id.get(&d.iri) {
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

                d.node.set_attribute((d.attr_id, av));
            }
        }
    }
}
