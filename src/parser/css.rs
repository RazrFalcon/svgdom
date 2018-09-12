// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use roxmltree;

use simplecss;
use simplecss::Token as CssToken;

use svgtypes::xmlparser::{
    Stream,
    StrSpan,
    TextPos,
};

use {
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
            Some(s) => StrSpan::from(s),
            None => continue,
        };

        if let Err(_) = parse_style(style, doc, post_data, &mut resolved_classes, opt) {
            let text_node = node.first_child().unwrap();
            // TODO: test
            let pos = ro_doc.text_pos_from(text_node.pos());
            return Err(ParserError::UnsupportedCSS(pos));
        }
    }

    postprocess_class_selector(&resolved_classes, &mut post_data.class_attrs, opt);

    Ok(())
}

fn parse_style(
    style: StrSpan,
    doc: &Document,
    post_data: &mut PostData,
    resolved_classes: &mut Vec<String>,
    opt: &ParseOptions,
) -> Result<(), ParserError> {
    let mut selectors: Vec<CssSelector> = Vec::new();
    let mut values: Vec<(&str,&str)> = Vec::with_capacity(16);

    let mut tokenizer = {
        let mut s = Stream::from(style);

        // check for a empty string
        s.skip_spaces();
        if s.at_end() {
            // ignore such CSS
            return Ok(())
        }

        // we use 'new_bound' method to get absolute error positions
        simplecss::Tokenizer::new(style.to_str())
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
                    return Err(ParserError::UnsupportedCSS(gen_err_pos(style, last_pos)));
                }
                _ => {
                    return Err(ParserError::InvalidCSS(gen_err_pos(style, last_pos)));
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
                    return Err(ParserError::InvalidCSS(gen_err_pos(style, last_pos)));
                }
            }
        }

        // process selectors
        for selector in &selectors {
            match *selector {
                CssSelector::Universal => {
                    for (_, mut node) in doc.root().descendants().svg() {
                        apply_css_attributes(&values, &mut node, &mut post_data.links, opt)?;
                    }
                }
                CssSelector::Type(name) => {
                    if let Some(eid) = ElementId::from_str(name) {
                        for (id, mut node) in doc.root().descendants().svg() {
                            if id == eid {
                                apply_css_attributes(&values, &mut node, &mut post_data.links, opt)?;
                            }
                        }
                    } else {
                        warn!("CSS styles for a non-SVG element ('{}') are ignored.", name);
                    }
                }
                CssSelector::Id(name) => {
                    if let Some(mut node) = doc.root().descendants().find(|n| *n.id() == name) {
                        apply_css_attributes(&values, &mut node, &mut post_data.links, opt)?;
                    }
                }
                CssSelector::Class(name) => {
                    // we use already collected list of 'class' attributes
                    for d in post_data.class_attrs.iter_mut().filter(|n| n.text == name) {
                        apply_css_attributes(&values, &mut d.node, &mut post_data.links, opt)?;

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
    values: &[(&str, &'a str)],
    node: &mut Node,
    links: &mut Links,
    opt: &ParseOptions,
) -> Result<(), ParserError> {
    for &(aname, avalue) in values {
        match AttributeId::from_str(aname) {
            Some(aid) => {
                let mut parse_attr = |aid: AttributeId| {
                    // TODO: to a proper stream
                    super::parse_svg_attribute_value(
                        aid, StrSpan::from(avalue), opt,
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
                    // So we have to convert it to presentation attributes.

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

fn gen_err_pos(span: StrSpan, pos: usize) -> TextPos {
    let s = Stream::from(span.full_str());
    s.gen_error_pos_from(pos)
}
