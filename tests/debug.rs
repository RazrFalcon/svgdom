// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] extern crate pretty_assertions;

extern crate svgdom;

use svgdom::{
    AttributeId as AId,
    Document,
    ElementId as EId,
    NodeType,
};

#[test]
fn elem_1() {
    let mut doc = Document::new();
    let svg_elem = doc.create_element(EId::Svg);

    assert_eq!(format!("{:?}", svg_elem), "Element(svg)");
    assert_eq!(format!("{}", svg_elem), "<svg>");
}

#[test]
fn elem_2() {
    let mut doc = Document::new();
    let mut svg_elem = doc.create_element(EId::Svg);
    svg_elem.set_attribute((AId::X, 1));

    assert_eq!(format!("{:?}", svg_elem), "Element(svg x=\"1\")");
    assert_eq!(format!("{}", svg_elem), "<svg x=\"1\">");
}

#[test]
fn elem_3() {
    let mut doc = Document::new();
    let mut svg_elem = doc.create_element(EId::Svg);
    svg_elem.set_id("svg1");
    svg_elem.set_attribute((AId::X, 1));
    svg_elem.set_attribute((AId::Y, 2));

    assert_eq!(format!("{:?}", svg_elem), "Element(svg id=\"svg1\" x=\"1\" y=\"2\")");
    assert_eq!(format!("{}", svg_elem), "<svg id=\"svg1\" x=\"1\" y=\"2\">");
}

#[test]
fn elem_4() {
    let mut doc = Document::new();
    let mut svg_elem = doc.create_element(EId::Svg);
    svg_elem.set_id("svg1");
    svg_elem.set_attribute((AId::X, 1));

    let mut lg_elem = doc.create_element(EId::LinearGradient);
    lg_elem.set_id("lg1");
    svg_elem.set_attribute((AId::Fill, lg_elem.clone()));

    assert_eq!(format!("{:?}", svg_elem), "Element(svg id=\"svg1\" x=\"1\" fill=\"url(#lg1)\")");
    assert_eq!(format!("{}", svg_elem), "<svg id=\"svg1\" x=\"1\" fill=\"url(#lg1)\">");

    assert_eq!(format!("{:?}", lg_elem), "Element(linearGradient id=\"lg1\"; linked-nodes: \"svg1\")");
    assert_eq!(format!("{}", lg_elem), "<linearGradient id=\"lg1\">");
}

#[test]
fn root_1() {
    let doc = Document::new();

    assert_eq!(format!("{:?}", doc.root()), "Root()");
    assert_eq!(format!("{}", doc.root()), "");
}

#[test]
fn comment_1() {
    let mut doc = Document::new();
    let node = doc.create_node(NodeType::Comment, "comment");

    assert_eq!(format!("{:?}", node), "Comment(comment)");
    assert_eq!(format!("{}", node), "<!--comment-->");
}

#[test]
fn text_1() {
    let mut doc = Document::new();
    let node = doc.create_node(NodeType::Text, "text");

    assert_eq!(format!("{:?}", node), "Text(text)");
    assert_eq!(format!("{}", node), "text");
}

#[test]
fn attributes_1() {
    let mut doc = Document::new();
    let mut svg_elem = doc.create_element(EId::Svg);
    svg_elem.set_id("svg1");
    svg_elem.set_attribute((AId::X, 1));
    svg_elem.set_attribute((AId::Y, 2));

    assert_eq!(format!("{:?}", svg_elem.attributes()), "Attributes(x=\"1\" y=\"2\")");
    assert_eq!(format!("{}", *svg_elem.attributes()), "x=\"1\" y=\"2\"");
}
