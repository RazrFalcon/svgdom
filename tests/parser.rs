#[macro_use] extern crate pretty_assertions;

use std::fmt;

use svgdom::{
    AttributeId as AId,
    AttributeValue,
    Document,
    ElementId as EId,
    TagNameRef,
    NodeType,
    ParseOptions,
    WriteOptions,
};

fn write_options() -> WriteOptions {
    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    opt
}

#[derive(Clone, Copy, PartialEq)]
struct TStr<'a>(pub &'a str);

impl<'a> fmt::Debug for TStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! test_resave {
    ($name:ident, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_str($in_text).unwrap();
            assert_eq!(TStr($out_text), TStr(doc.to_string_with_opt(&write_options()).as_str()));
        }
    )
}

#[test]
fn parse_empty_1() {
    assert_eq!(Document::from_str("").err().unwrap().to_string(),
        "the document does not have a root node");
}

#[test]
fn parse_empty_2() {
    assert_eq!(Document::from_str("\n \t").err().unwrap().to_string(),
        "the document does not have a root node");
}

#[test]
fn parse_empty_3() {
    assert_eq!(Document::from_str("<rect/>").err().unwrap().to_string(),
        "the document does not have an SVG element");
}

#[test]
fn parse_empty_4() {
    assert_eq!(Document::from_str("<?xml version='1.0'?>").err().unwrap().to_string(),
        "the document does not have a root node");
}

#[test]
fn parse_single_node_1() {
    let doc = Document::from_str("<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();

    let child = doc.root().first_child().unwrap();
    assert_eq!(child.tag_name().as_ref(), TagNameRef::from(EId::Svg));
    assert_eq!(doc.root().children().count(), 1);
}

#[test]
fn parse_comment_1() {
    let doc = Document::from_str("<svg xmlns='http://www.w3.org/2000/svg'/><!--comment-->").unwrap();

    let child = doc.root().children().nth(1).unwrap();
    assert_eq!(child.node_type(), NodeType::Comment);
    assert_eq!(*child.text(), "comment");
    assert_eq!(doc.root().children().count(), 2);
}

#[test]
fn parse_text_1() {
    let doc = Document::from_str("<svg xmlns='http://www.w3.org/2000/svg'>text</svg>").unwrap();

    let child = doc.root().first_child().unwrap().first_child().unwrap();
    assert_eq!(child.node_type(), NodeType::Text);
    assert_eq!(*child.text(), "text");
}

#[test]
fn parse_text_2() {
    let doc = Document::from_str("<svg xmlns='http://www.w3.org/2000/svg'><text>Some<tspan>complex</tspan>text</text></svg>").unwrap();

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

test_resave!(parse_non_svg_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <qwe/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

test_resave!(parse_non_svg_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect qwe='qwe'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>
");

// style must be ungroupped after presentation attributes
test_resave!(parse_style_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g style='fill:green' fill='red'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#008000'/>
</svg>
");

// style must be ungroupped after presentation attributes
test_resave!(parse_style_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g style='fill:none; color:cyan; stroke-width:4.00'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g color='#00ffff' fill='none' stroke-width='4'/>
</svg>
");

// style must be ungroupped after presentation attributes
test_resave!(parse_style_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text style=\"font-size:24px;font-style:normal;font-variant:normal;font-weight:normal;\
                  font-stretch:normal;line-height:125%;writing-mode:lr-tb;\
                  text-anchor:middle;font-family:'Arial Bold'\"/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text font-family='Arial Bold' font-size='24px' font-stretch='normal' \
                   font-style='normal' font-variant='normal' font-weight='normal' \
                   line-height='125%' text-anchor='middle' \
                   writing-mode='lr-tb'/>
</svg>
");

// comments inside attribute are ignored
test_resave!(parse_style_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text style='font-size:24px; /* comment */ font-style:normal;'/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text font-size='24px' font-style='normal'/>
</svg>
");

// all attributes must begin with a letter
test_resave!(parse_style_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text style='font-size:24px;-font-style:normal;font-stretch:normal;'/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text font-size='24px' font-stretch='normal'/>
</svg>
");

// keep unknown attributes
test_resave!(parse_style_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g style='qwe:none; color:cyan;'/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g color='#00ffff' qwe='none'/>
</svg>
");

#[test]
fn parse_paint_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
    <radialGradient id='0-5'/>
    <rect fill='url(#0-5)'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <radialGradient id='0-5'/>
    <rect fill='url(#0-5)'/>
</svg>
");

test_resave!(parse_paint_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='url(#lg1) none'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='none'/>
</svg>
");

test_resave!(parse_paint_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='url(#lg1) red'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='#ff0000'/>
</svg>
");

test_resave!(parse_paint_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='url(#lg1) currentColor'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='currentColor'/>
</svg>
");

test_resave!(parse_paint_7,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient id='lg1'/>
    <rect fill='url(#lg1) none'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient id='lg1'/>
    <rect fill='url(#lg1) none'/>
</svg>
");

test_resave!(parse_invalid_path,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <path d='M 10 20 L 30 40 L 50'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <path d='M 10 20 L 30 40'/>
</svg>
");

#[test]
fn parse_iri_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <rect id='r1'/>
    <use xlink:href='#r1'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let rect_node = svg_node.children().nth(0).unwrap();
    let use_node = svg_node.children().nth(1).unwrap();

    assert_eq!(rect_node.is_used(), true);
    assert_eq!(use_node.attributes().get_value(AId::Href).unwrap(),
               &AttributeValue::Link(rect_node));
}

#[test]
fn parse_iri_2() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <use xlink:href='#r1'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let use_node = svg_node.children().nth(0).unwrap();

    assert_eq!(use_node.attributes().get_value(AId::Href).unwrap(),
               &AttributeValue::String("#r1".to_string()));
}

#[test]
fn parse_func_iri_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect filter='url(#f)'/>
</svg>").unwrap();

    let svg_node = doc.root().first_child().unwrap();
    let rect_node = svg_node.children().nth(0).unwrap();

    assert_eq!(rect_node.attributes().get_value(AId::Filter).unwrap(),
               &AttributeValue::String("url(#f)".to_string()));
}

// ignore empty LengthList
test_resave!(parse_empty_attribute_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect stroke-dasharray=''/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>
");

// ignore empty NumberList
test_resave!(parse_empty_attribute_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect stdDeviation=''/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>
");

// ignore empty Transform
test_resave!(parse_empty_attribute_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect transform=''/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>
");

test_resave!(parse_viewbox_1,
"<svg xmlns='http://www.w3.org/2000/svg' viewBox='10 20 30 40'/>",
"<svg xmlns='http://www.w3.org/2000/svg' viewBox='10 20 30 40'/>
");

#[test]
fn skip_unresolved_classes_1() {
    let mut opt = ParseOptions::default();
    opt.skip_unresolved_classes = false;
    let doc = Document::from_str_with_opt(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
        .fil1 {fill:blue}
        .str1 {stroke:blue}
    </style>
    <g class='fil1 fil3'/>
    <g class='fil1 fil4 str1 fil5'/>
</svg>", &opt).unwrap();

    assert_eq!(doc.to_string_with_opt(&write_options()),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g class='fil3' fill='#0000ff'/>
    <g class='fil4 fil5' fill='#0000ff' stroke='#0000ff'/>
</svg>
");
}

test_resave!(crosslink_1,
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <linearGradient id='lg1' xlink:href='#lg2'/>
    <linearGradient id='lg2' xlink:href='#lg1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <linearGradient id='lg1' xlink:href='#lg2'/>
    <linearGradient id='lg2'/>
</svg>
");

test_resave!(crosslink_2,
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <linearGradient id='lg1' xlink:href='#lg1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient id='lg1'/>
</svg>
");

// Checks that deep recursion doesn't cause a memory leak.
test_resave!(crosslink_3,
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <linearGradient id='lg1' xlink:href='#lg2'/>
    <linearGradient id='lg2' xlink:href='#lg3'/>
    <linearGradient id='lg3' xlink:href='#lg1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink'>
    <linearGradient id='lg1' xlink:href='#lg2'/>
    <linearGradient id='lg2' xlink:href='#lg3'/>
    <linearGradient id='lg3' xlink:href='#lg1'/>
</svg>
");

#[test]
fn attr_value_error_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='qwe'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 2:17");
}

#[test]
fn attr_value_error_2() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect style='fill:qwe'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 2:18");
}

#[test]
fn attr_value_error_3() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect stroke-miterlimit='5mm'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 2:30");
}

#[test]
fn attr_value_error_4() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect opacity='5mm'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 2:20");
}

#[test]
fn attr_value_error_5() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect offset='5%'/> <!-- % is ok -->
    <rect offset='5mm'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 3:19");
}

#[test]
fn attr_value_error_6() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect width='5mmx'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "invalid attribute value at 2:18");
}

// TODO: this
// p { font-family: "Font 1", "Font 2", Georgia, Times, serif; }
