// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use log::warn;

use roxmltree::{
    self,
    TextPos,
};

use simplecss;
use simplecss::Token as CssToken;

use svgtypes::Stream;

use crate::{
    AttributeId,
    AttributeValue,
    Document,
    ElementId,
    FilterSvg,
    Node,
    ParseOptions,
    ParserError,
};

use super::{
    Links,
    NodeStringData,
    PostData,
};

#[derive(Clone, Copy, Debug)]
enum CssSelector<'a> {
    Universal,
    Type(&'a str),
    Id(&'a str),
    Class(&'a str),
}

pub fn resolve_css(
    ro_doc: &roxmltree::Document,
    doc: &Document,
    post_data: &mut PostData,
    opt: &ParseOptions,
) -> Result<(), ParserError> {
    // remember all resolved classes
    let mut resolved_classes: Vec<String> = Vec::with_capacity(16);

    for node in ro_doc.descendants().filter(|n| n.has_tag_name("style")) {
        match node.attribute("type") {
            Some("text/css") => {}
            None => {}
            Some(_) => continue,
        }

        let style = match node.text() {
            Some(s) => s,
            None => continue,
        };

        if let Err(_) = parse_style(ro_doc, style, doc, post_data, &mut resolved_classes, opt) {
            if opt.skip_invalid_css {
                warn!("Document contains an unsupported CSS.");
            } else {
                // If an error occurred then use the text node position.
                let text_node = node.first_child().unwrap();
                let pos = ro_doc.text_pos_at(text_node.range().start);
                return Err(ParserError::UnsupportedCSS(pos));
            }
        }
    }

    postprocess_class_selector(&resolved_classes, &mut post_data.class_attrs, opt);

    Ok(())
}

fn parse_style(
    ro_doc: &roxmltree::Document,
    style: &str,
    doc: &Document,
    post_data: &mut PostData,
    resolved_classes: &mut Vec<String>,
    opt: &ParseOptions,
) -> Result<(), ParserError> {
    let mut selectors: Vec<CssSelector> = Vec::new();
    let mut values: Vec<(&str,&str)> = Vec::with_capacity(16);

    // Position doesn't matter, because we ignore this errors anyway.
    macro_rules! gen_err {
        () => { ParserError::UnsupportedCSS(TextPos::new(1, 1)) };
    }

    let mut tokenizer = {
        let mut s = Stream::from(style);

        // check for a empty string
        s.skip_spaces();
        if s.at_end() {
            // ignore such CSS
            return Ok(());
        }

        // we use 'new_bound' method to get absolute error positions
        simplecss::Tokenizer::new(style)
    };

    'root: loop {
        selectors.clear();
        values.clear();

        // get list of selectors
        loop {
            let token = tokenizer.parse_next().map_err(|_| gen_err!())?;

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
                _ => return Err(gen_err!()),
            };

            selectors.push(selector);
        }

        // get list of declarations
        loop {
            match tokenizer.parse_next().map_err(|_| gen_err!())? {
                CssToken::Declaration(name, value) => values.push((name, value)),
                CssToken::BlockEnd => break,
                CssToken::EndOfStream => break 'root,
                _ => return Err(gen_err!()),
            }
        }

        // process selectors
        for selector in &selectors {
            match *selector {
                CssSelector::Universal => {
                    for (_, mut node) in doc.root().descendants().svg() {
                        apply_css_attributes(ro_doc, &values, opt,
                                             &mut node, &mut post_data.links)?;
                    }
                }
                CssSelector::Type(name) => {
                    if let Some(eid) = ElementId::from_str(name) {
                        for (id, mut node) in doc.root().descendants().svg() {
                            if id == eid {
                                apply_css_attributes(ro_doc, &values, opt,
                                                     &mut node, &mut post_data.links)?;
                            }
                        }
                    } else {
                        warn!("CSS styles for a non-SVG element ('{}') are ignored.", name);
                    }
                }
                CssSelector::Id(name) => {
                    if let Some(mut node) = doc.root().descendants().find(|n| *n.id() == name) {
                        apply_css_attributes(ro_doc, &values, opt,
                                             &mut node, &mut post_data.links)?;
                    }
                }
                CssSelector::Class(name) => {
                    // we use already collected list of 'class' attributes
                    for d in post_data.class_attrs.iter_mut().filter(|n| n.text == name) {
                        apply_css_attributes(ro_doc, &values, opt,
                                             &mut d.node, &mut post_data.links)?;

                        resolved_classes.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(())
}

fn postprocess_class_selector<'a>(
    resolved_classes: &[String],
    class_attrs: &mut Vec<NodeStringData>,
    opt: &ParseOptions,
) {
    // remove resolved classes
    class_attrs.retain(|n| !resolved_classes.contains(&n.text));

    if opt.skip_unresolved_classes {
        for d in class_attrs {
            warn!("Could not resolve an unknown class: {}.", d.text);
        }
    } else {
        // create 'class' attributes with unresolved classes
        for d in class_attrs {
            if d.node.has_attribute(AttributeId::Class) {
                let mut attrs = d.node.attributes_mut();
                let class_val = attrs.get_value_mut(AttributeId::Class);
                if let Some(&mut AttributeValue::String(ref mut text)) = class_val {
                    text.push(' ');
                    text.push_str(&d.text);
                }
            } else {
                d.node.set_attribute((AttributeId::Class, d.text.clone()));
            }
        }
    }
}

fn apply_css_attributes<'a>(
    ro_doc: &roxmltree::Document,
    values: &[(&str, &'a str)],
    opt: &ParseOptions,
    node: &mut Node,
    links: &mut Links,
) -> Result<(), ParserError> {
    for &(aname, avalue) in values {
        match AttributeId::from_str(aname) {
            Some(aid) => {
                let mut parse_attr = |aid: AttributeId| {
                    super::parse_svg_attribute_value(
                        ro_doc, aid, avalue, 0, opt,
                        node, links,
                    )
                };

                if aid == AttributeId::Marker {
                    // The SVG specification defines three properties to reference markers:
                    // `marker-start`, `marker-mid`, `marker-end`.
                    // It also provides a shorthand property, marker.
                    // Using the marker property from a style sheet
                    // is equivalent to using all three (start, mid, end).
                    // However, shorthand properties cannot be used as presentation attributes.
                    // So we have to convert it into presentation attributes.

                    parse_attr(AttributeId::MarkerStart)?;
                    parse_attr(AttributeId::MarkerMid)?;
                    parse_attr(AttributeId::MarkerEnd)?;
                } else {
                    parse_attr(aid)?;
                }
            }
            None => {
                node.set_attribute((aname, avalue));
            }
        }
    }

    Ok(())
}
