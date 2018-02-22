// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use simplecss;

use svgparser::{
    Stream,
    StrSpan,
};

use error::Result;
use {
    AttributeId,
    AttributeValue,
    Document,
    ElementId,
    ErrorKind,
    ErrorPos,
    Node,
    ParseOptions,
};

use super::parser::{
    Entities,
    Links,
    NodeSpanData,
    PostData,
};


pub fn resolve_css<'a>(
    doc: &Document,
    post_data: &mut PostData<'a>,
    opt: &ParseOptions,
) -> Result<()> {
    use simplecss::Token as CssToken;

    #[derive(Clone,Copy,Debug)]
    enum CssSelector<'a> {
        Universal,
        Type(&'a str),
        Id(&'a str),
        Class(&'a str),
    }

    fn gen_err_pos(frame: StrSpan, pos: usize) -> ErrorPos {
        let mut s = Stream::from_str(frame.full_str());
        s.gen_error_pos_from(pos)
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
            let mut s = Stream::from_span(*style);

            // check for a empty string
            s.skip_spaces();
            if s.at_end() {
                // ignore such CSS
                continue;
            }

            let text = style.full_str();

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
                        return Err(ErrorKind::UnsupportedCSS(gen_err_pos(*style, last_pos)).into());
                    }
                    _ => {
                        return Err(ErrorKind::InvalidCSS(gen_err_pos(*style, last_pos)).into());
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
                        return Err(ErrorKind::InvalidCSS(gen_err_pos(*style, last_pos)).into());
                    }
                }
            }

            // process selectors
            for selector in &selectors {
                match *selector {
                    CssSelector::Universal => {
                        for (_, mut node) in doc.descendants().svg() {
                            apply_css_attributes(&values, &mut node, &mut post_data.links,
                                                 &post_data.entitis, opt)?;
                        }
                    }
                    CssSelector::Type(name) => {
                        if let Some(eid) = ElementId::from_name(name) {
                            for (id, mut node) in doc.descendants().svg() {
                                if id == eid {
                                    apply_css_attributes(&values, &mut node, &mut post_data.links,
                                                         &post_data.entitis, opt)?;
                                }
                            }
                        } else {
                            warn!("CSS styles for a non-SVG element ('{}') are ignored.", name);
                        }
                    }
                    CssSelector::Id(name) => {
                        if let Some(mut node) = doc.descendants().find(|n| *n.id() == name) {
                            apply_css_attributes(&values, &mut node, &mut post_data.links,
                                                 &post_data.entitis, opt)?;
                        }
                    }
                    CssSelector::Class(name) => {
                        // we use already collected list of 'class' attributes
                        for d in post_data.class_attrs.iter_mut().filter(|n| n.span.to_str() == name) {
                            apply_css_attributes(&values, &mut d.node, &mut post_data.links,
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

fn postprocess_class_selector<'a>(
    resolved_classes: &[&str],
    class_attrs: &mut Vec<NodeSpanData<'a>>,
    opt: &ParseOptions,
) {
    // remove resolved classes
    class_attrs.retain(|n| !resolved_classes.contains(&n.span.to_str()));

    if opt.skip_unresolved_classes {
        for d in class_attrs {
            warn!("Could not resolve an unknown class: {}.", d.span);
        }
    } else {
        // create 'class' attributes with unresolved classes
        for d in class_attrs {
            if d.node.has_attribute(AttributeId::Class) {
                let mut attrs = d.node.attributes_mut();
                let class_val = attrs.get_value_mut(AttributeId::Class);
                if let Some(&mut AttributeValue::String(ref mut text)) = class_val {
                    text.push(' ');
                    text.push_str(d.span.to_str());
                }
            } else {
                d.node.set_attribute((AttributeId::Class, d.span.to_str()));
            }
        }
    }
}

fn apply_css_attributes<'a>(
    values: &[(&str, &'a str)],
    node: &mut Node,
    links: &mut Links<'a>,
    entitis: &Entities<'a>,
    opt: &ParseOptions,
) -> Result<()> {
    for &(aname, avalue) in values {
        match AttributeId::from_name(aname) {
            Some(aid) => {
                let mut parse_attr = |aid: AttributeId| {
                    // TODO: to a proper stream
                    super::parser::parse_svg_attribute_value(
                        node, "", aid, StrSpan::from_str(avalue),
                        links, entitis, opt
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
                if opt.parse_unknown_attributes {
                    node.set_attribute((aname, avalue));
                }
            }
        }
    }

    Ok(())
}
