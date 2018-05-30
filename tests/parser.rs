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
    AttributeValue,
    Document,
    ElementId as EId,
    TagNameRef,
    NodeType,
    ParseOptions,
    WriteBuffer,
    WriteOptions,
};

fn write_options() -> WriteOptions {
    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    opt
}

macro_rules! test_resave {
    ($name:ident, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_str($in_text).unwrap();
            assert_eq!(doc.with_write_opt(&write_options()).to_string(), $out_text);
        }
    )
}

#[test]
fn parse_empty_1() {
    assert_eq!(Document::from_str("").err().unwrap().to_string(),
        "the document does not have any nodes");
}

#[test]
fn parse_empty_2() {
    assert_eq!(Document::from_str("\n \t").err().unwrap().to_string(),
        "the document does not have any nodes");
}

#[test]
fn parse_empty_3() {
    assert_eq!(Document::from_str("<rect/>").err().unwrap().to_string(),
        "the document does not have an SVG element");
}

#[test]
fn parse_empty_4() {
    assert_eq!(Document::from_str("<?xml version='1.0'?>").err().unwrap().to_string(),
        "the document does not have an SVG element");
}

#[test]
fn parse_single_node_1() {
    let doc = Document::from_str("<svg/>").unwrap();

    let child = doc.root().first_child().unwrap();
    assert_eq!(child.tag_name().as_ref(), TagNameRef::from(EId::Svg));
    assert_eq!(doc.root().children().count(), 1);
}

#[test]
fn parse_declaration_1() {
    let doc = Document::from_str(
"<?xml version='1.0' encoding='UTF-8' standalone='yes'?>
<svg/>"
    ).unwrap();

    let decl = doc.root().first_child().unwrap();
    assert!(decl.is_declaration());

    let attrs = decl.attributes();
    assert_eq!(attrs.len(), 3);
    assert_eq!(attrs.get_value(AId::Version), Some(&AttributeValue::String("1.0".to_string())));
    assert_eq!(attrs.get_value(AId::Encoding), Some(&AttributeValue::String("UTF-8".to_string())));
    assert_eq!(attrs.get_value(AId::Standalone), Some(&AttributeValue::String("yes".to_string())));
}

#[test]
fn parse_comment_1() {
    let doc = Document::from_str("<svg/><!--comment-->").unwrap();

    let child = doc.root().children().nth(1).unwrap();
    assert_eq!(child.node_type(), NodeType::Comment);
    assert_eq!(*child.text(), "comment");
    assert_eq!(doc.root().children().count(), 2);
}

#[test]
fn parse_text_1() {
    let doc = Document::from_str("<svg>text</svg>").unwrap();

    let child = doc.root().first_child().unwrap().first_child().unwrap();
    assert_eq!(child.node_type(), NodeType::Text);
    assert_eq!(*child.text(), "text");
}

#[test]
fn parse_text_2() {
    let doc = Document::from_str("<svg><text>Some<tspan>complex</tspan>text</text></svg>").unwrap();

    let mut nodes = doc.root().first_child().unwrap().descendants();

    let svg_node = nodes.next().unwrap();
    assert_eq!(svg_node.tag_name().as_ref(), TagNameRef::from(EId::Svg));
    assert_eq!(svg_node.node_type(), NodeType::Element);

    let text_node = nodes.next().unwrap();
    assert_eq!(text_node.tag_name().as_ref(), TagNameRef::from(EId::Text));
    assert_eq!(text_node.node_type(), NodeType::Element);

    let text_data_node = nodes.next().unwrap();
    assert_eq!(*text_data_node.text(), "Some");
    assert_eq!(text_data_node.node_type(), NodeType::Text);

    let tspan_node = nodes.next().unwrap();
    assert_eq!(tspan_node.tag_name().as_ref(), TagNameRef::from(EId::Tspan));
    assert_eq!(tspan_node.node_type(), NodeType::Element);

    let text_data_node_2 = nodes.next().unwrap();
    assert_eq!(*text_data_node_2.text(), "complex");
    assert_eq!(text_data_node_2.node_type(), NodeType::Text);

    let text_data_node_3 = nodes.next().unwrap();
    assert_eq!(*text_data_node_3.text(), "text");
    assert_eq!(text_data_node_3.node_type(), NodeType::Text);
}

// style must be ungroupped after presentation attributes
test_resave!(parse_style_1,
"<svg>
    <g style='fill:green' fill='red'/>
</svg>",
"<svg>
    <g fill='#008000'/>
</svg>
");

// style must be ungroupped after presentation attributes
test_resave!(parse_style_2,
"<svg>
    <g style='fill:none; color:cyan; stroke-width:4.00'/>
</svg>",
"<svg>
    <g color='#00ffff' fill='none' stroke-width='4'/>
</svg>
");

// style must be ungroupped after presentation attributes
test_resave!(parse_style_3,
"<svg>
    <text style=\"font-size:24px;font-style:normal;font-variant:normal;font-weight:normal;\
                  font-stretch:normal;line-height:125%;writing-mode:lr-tb;\
                  text-anchor:middle;font-family:'Arial Bold'\"/>
</svg>
",
"<svg>
    <text font-family='Arial Bold' font-size='24px' font-stretch='normal' \
                   font-style='normal' font-variant='normal' font-weight='normal' \
                   line-height='125%' text-anchor='middle' \
                   writing-mode='lr-tb'/>
</svg>
");

// comments inside attribute are ignored
test_resave!(parse_style_4,
"<svg>
    <text style='font-size:24px; /* comment */ font-style:normal;'/>
</svg>
",
"<svg>
    <text font-size='24px' font-style='normal'/>
</svg>
");

// all attributes must begin with a letter
test_resave!(parse_style_5,
"<svg>
    <text style='font-size:24px;-font-style:normal;font-stretch:normal;'/>
</svg>
",
"<svg>
    <text font-size='24px' font-stretch='normal'/>
</svg>
");

// keep unknown attributes
test_resave!(parse_style_6,
"<svg>
    <g style='qwe:none; color:cyan;'/>
</svg>
",
"<svg>
    <g color='#00ffff' qwe='none'/>
</svg>
");

// remove unknown linked styles
test_resave!(parse_style_8,
"<svg>
    <g style='&st0; &st1;'/>
</svg>
",
"<svg>
    <g/>
</svg>
");

#[test]
fn parse_paint_1() {
    let doc = Document::from_str(
"<svg>
    <radialGradient id='rg1'/>
    <rect fill='url(#rg1)'/>
</svg>").unwrap();

    let child = doc.root().first_child().unwrap();
    let rg = child.children().nth(0).unwrap();
    let rect = child.children().nth(1).unwrap();

    assert_eq!(rg.is_used(), true);
    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(), &AttributeValue::Paint(rg, None));
}

#[test]
fn parse_paint_2() {
    // reversed order

    let doc = Document::from_str(
"<svg>
    <rect fill='url(#rg1)'/>
    <radialGradient id='rg1'/>
</svg>").unwrap();

    let child = doc.root().first_child().unwrap();
    let rect = child.children().nth(0).unwrap();
    let rg = child.children().nth(1).unwrap();

    assert_eq!(rg.is_used(), true);
    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(), &AttributeValue::Paint(rg, None));
}

test_resave!(parse_paint_3,
"<svg>
    <radialGradient id='0-5'/>
    <rect fill='url(#0-5)'/>
</svg>",
"<svg>
    <radialGradient id='0-5'/>
    <rect fill='url(#0-5)'/>
</svg>
");

test_resave!(parse_paint_4,
"<svg>
    <rect fill='url(#lg1) none'/>
</svg>",
"<svg>
    <rect fill='none'/>
</svg>
");

test_resave!(parse_paint_5,
"<svg>
    <rect fill='url(#lg1) red'/>
</svg>",
"<svg>
    <rect fill='#ff0000'/>
</svg>
");

test_resave!(parse_paint_6,
"<svg>
    <rect fill='url(#lg1) currentColor'/>
</svg>",
"<svg>
    <rect fill='currentColor'/>
</svg>
");

test_resave!(parse_paint_7,
"<svg>
    <linearGradient id='lg1'/>
    <rect fill='url(#lg1) none'/>
</svg>",
"<svg>
    <linearGradient id='lg1'/>
    <rect fill='url(#lg1) none'/>
</svg>
");

#[test]
fn parse_iri_1() {
    let doc = Document::from_str(
"<svg>
    <rect id='r1'/>
    <use xlink:href='#r1'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let rect_node = svg_node.children().nth(0).unwrap();
    let use_node = svg_node.children().nth(1).unwrap();

    assert_eq!(rect_node.is_used(), true);
    assert_eq!(use_node.attributes().get_value(("xlink", AId::Href)).unwrap(),
               &AttributeValue::Link(rect_node));
}

#[test]
fn parse_iri_2() {
    let doc = Document::from_str(
"<svg>
    <use xlink:href='#r1'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let use_node = svg_node.children().nth(0).unwrap();

    assert_eq!(use_node.attributes().get_value(("xlink", AId::Href)).unwrap(),
               &AttributeValue::String("#r1".to_string()));
}

#[test]
fn parse_func_iri_1() {
    let doc = Document::from_str(
"<svg>
    <filter id='f'/>
    <rect filter='url(#f)'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let filter_node = svg_node.children().nth(0).unwrap();
    let rect_node = svg_node.children().nth(1).unwrap();

    assert_eq!(filter_node.is_used(), true);
    assert_eq!(rect_node.attributes().get_value(AId::Filter).unwrap(),
               &AttributeValue::FuncLink(filter_node));
}

#[test]
fn parse_func_iri_2() {
    let doc = Document::from_str(
"<svg>
    <rect filter='url(#f)'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let rect_node = svg_node.children().nth(0).unwrap();

    assert_eq!(rect_node.attributes().get_value(AId::Filter).unwrap(),
               &AttributeValue::String("url(#f)".to_string()));
}

test_resave!(parse_entity_1,
"<!DOCTYPE svg [
    <!ENTITY st1 \"font-size:12;\">
]>
<svg style='&st1;'/>",
"<svg font-size='12'/>
");

// inside svg attribute
test_resave!(parse_entity_2,
"<!DOCTYPE svg [
    <!ENTITY ns_svg \"http://www.w3.org/2000/svg\">
    <!ENTITY ns_xlink \"http://www.w3.org/1999/xlink\">
]>
<svg xmlns='&ns_svg;' xmlns:xlink='&ns_xlink;'/>",
"<svg xmlns:xlink='http://www.w3.org/1999/xlink' xmlns='http://www.w3.org/2000/svg'/>
");

// inside external attribute
test_resave!(parse_entity_3,
"<!DOCTYPE svg [
    <!ENTITY ns_extend \"http://ns.adobe.com/Extensibility/1.0/\">
]>
<svg xmlns:x='&ns_extend;'/>",
"<svg xmlns:x='http://ns.adobe.com/Extensibility/1.0/'/>
");

test_resave!(parse_entity_4,
"<!DOCTYPE svg [
    <!ENTITY st1 \"red\">
]>
<svg fill='&st1;'/>",
"<svg fill='#ff0000'/>
");

test_resave!(skip_unknown_refs_1,
"<svg unicode='&#x3b2;'/>",
"<svg unicode='&#x3b2;'/>
");

// ignore empty LengthList
test_resave!(parse_empty_attribute_1,
"<svg>
    <rect stroke-dasharray=''/>
</svg>",
"<svg>
    <rect/>
</svg>
");

// ignore empty NumberList
test_resave!(parse_empty_attribute_2,
"<svg>
    <rect stdDeviation=''/>
</svg>",
"<svg>
    <rect/>
</svg>
");

// ignore empty Transform
test_resave!(parse_empty_attribute_3,
"<svg>
    <rect transform=''/>
</svg>",
"<svg>
    <rect/>
</svg>
");

test_resave!(parse_script_1,
"<svg>
    <script><![CDATA[
        var i, ids = 'a1 a2 a3 a4 a5 a6 r1 r2 r3 r4 r5 r6 r7 r8'.split(' ');
        for (i in ids) {
            this[ids[i]] = document.getElementById(ids[i]);
        }
    ]]></script>
</svg>",
"<svg>
    <script>
    <![CDATA[
        var i, ids = 'a1 a2 a3 a4 a5 a6 r1 r2 r3 r4 r5 r6 r7 r8'.split(' ');
        for (i in ids) {
            this[ids[i]] = document.getElementById(ids[i]);
        }
    ]]>
    </script>
</svg>
");

test_resave!(parse_viewbox_1,
"<svg viewBox='10 20 30 40'/>",
"<svg viewBox='10 20 30 40'/>
");

#[test]
fn skip_unresolved_classes_1() {
    let mut opt = ParseOptions::default();
    opt.skip_unresolved_classes = false;
    let doc = Document::from_str_with_opt(
"<svg>
    <style type='text/css'>
        .fil1 {fill:blue}
        .str1 {stroke:blue}
    </style>
    <g class='fil1 fil3'/>
    <g class='fil1 fil4 str1 fil5'/>
</svg>", &opt).unwrap();

    assert_eq!(doc.with_write_opt(&write_options()).to_string(),
"<svg>
    <g class='fil3' fill='#0000ff'/>
    <g class='fil4 fil5' fill='#0000ff' stroke='#0000ff'/>
</svg>
");
}

test_resave!(elements_from_entity_1,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect width='10' height='20' fill='none'/>\">
]>
<svg>&Rect1;</svg>",
"<svg>
    <rect fill='none' height='20' width='10'/>
</svg>
");

test_resave!(elements_from_entity_2,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect width='10' height='20' fill='none'/>\">
]>
<svg>&Rect1;&Rect1;&Rect1;</svg>",
"<svg>
    <rect fill='none' height='20' width='10'/>
    <rect fill='none' height='20' width='10'/>
    <rect fill='none' height='20' width='10'/>
</svg>
");

test_resave!(elements_from_entity_3,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect width='10' height='20' fill='none'/>\">
]>
<svg>&Rect1; some text &Rect1;</svg>",
"<svg>
    <rect fill='none' height='20' width='10'/>
    <rect fill='none' height='20' width='10'/>
</svg>
");

// Keep unresolved references.
// TODO: wrong, do not convert '&' to '&amp;'
test_resave!(elements_from_entity_4,
"<svg>&Rect1;</svg>",
"<svg>&amp;Rect1;</svg>
");

// Resolve references only inside the container-based elements.
test_resave!(elements_from_entity_5,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect width='10' height='20' fill='none'/>\">
]>
<svg>
    <text>&Rect1;</text>
</svg>",
"<svg>
    <text>&amp;Rect1;</text>
</svg>
");

test_resave!(elements_from_entity_6,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
<!ENTITY Rect1 \"
    <g>
        <rect width='10' height='20' fill='none'/>
    </g>
\">
]>
<svg>
    &Rect1;
</svg>",
"<svg>
    <g>
        <rect fill='none' height='20' width='10'/>
    </g>
</svg>
");

test_resave!(elements_from_entity_7,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect>\">
]>
<svg>&Rect1;</svg>",
"<svg>
    <rect/>
</svg>
");

#[test]
fn elements_from_entity_8() {
    let doc = Document::from_str(
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"</rect>\">
]>
<svg>&Rect1;</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "opening and ending tag mismatch 'svg' and 'rect'");
}

test_resave!(elements_from_entity_9,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect/><rect/>\">
]>
<svg>&Rect1;</svg>",
"<svg>
    <rect/>
    <rect/>
</svg>
");

test_resave!(elements_from_entity_10,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
    <!ENTITY Rect1 \"<rect width='10' height='20' fill='none'/>\">
]>
<svg>
    <g>&Rect1;</g>
    <g>&Rect1;</g>
</svg>",
"<svg>
    <g>
        <rect fill='none' height='20' width='10'/>
    </g>
    <g>
        <rect fill='none' height='20' width='10'/>
    </g>
</svg>
");

test_resave!(elements_from_entity_11,
"<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1 Basic//EN\" \"http://www.w3.org/\" [
<!ENTITY Rect1 \"
    <rect id='rect1' width='10' height='20' fill='none'/>
    <use xlink:href='#rect1'/>
\">
]>
<svg>
    &Rect1;
</svg>",
"<svg>
    <rect id='rect1' fill='none' height='20' width='10'/>
    <use xlink:href='#rect1'/>
</svg>
");

#[test]
fn unbalanced_tree_1() {
    let doc = Document::from_str(
"<svg>
    </rect>
</svg>");
    assert_eq!(doc.err().unwrap().to_string(),
               "opening and ending tag mismatch 'svg' and 'rect'");
}

#[test]
fn unbalanced_tree_2() {
    let doc = Document::from_str(
"<svg:svg>
    </svg:rect>
</svg>");
    assert_eq!(doc.err().unwrap().to_string(),
               "opening and ending tag mismatch 'svg:svg' and 'svg:rect'");
}

#[test]
fn unbalanced_tree_3() {
    let doc = Document::from_str(
"<svg>
    <rect>
</svg>");
    assert_eq!(doc.err().unwrap().to_string(),
               "opening and ending tag mismatch 'rect' and 'svg'");
}

#[test]
fn unbalanced_tree_4() {
    let doc = Document::from_str("</svg>");
    assert_eq!(doc.err().unwrap().to_string(),
               "unexpected token 'Element Close' at 1:1");
}

// TODO: this
// p { font-family: "Font 1", "Font 2", Georgia, Times, serif; }
