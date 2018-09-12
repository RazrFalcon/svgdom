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
    AttributesOrder,
    Color,
    Document,
    ElementId as EId,
    Indent,
    Length,
    LengthUnit,
    NodeType,
    Transform,
    ViewBox,
    WriteOptions,
    WriteBuffer,
    NumberList,
    LengthList,
};

macro_rules! test_resave {
    ($name:ident, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_str($in_text).unwrap();

            let mut opt = WriteOptions::default();
            opt.use_single_quote = true;

            assert_eq!(doc.with_write_opt(&opt).to_string(), $out_text);
        }
    )
}

#[test]
fn empty_doc_1() {
    assert_eq!(Document::new().to_string(), String::new());
}

#[test]
fn single_node_1() {
    let mut doc = Document::new();
    let n = doc.create_element(EId::Svg);

    doc.root().append(n.clone());

    assert_eq!(doc.to_string(), "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n");
}

#[test]
fn child_node_1() {
    let mut doc = Document::new();
    let mut svg = doc.create_element(EId::Svg);
    let defs = doc.create_element(EId::Defs);

    doc.root().append(svg.clone());
    svg.append(defs.clone());

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\">
    <defs/>
</svg>
");
}

#[test]
fn child_nodes_1() {
    let mut doc = Document::new();
    let svg = doc.create_element(EId::Svg);
    doc.root().append(svg.clone());

    let mut parent = svg;
    for n in 1..5 {
        let mut r = doc.create_element(EId::Rect);
        r.set_id(n.to_string());
        parent.append(r.clone());

        parent = r;
    }

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\">
    <rect id=\"1\">
        <rect id=\"2\">
            <rect id=\"3\">
                <rect id=\"4\"/>
            </rect>
        </rect>
    </rect>
</svg>
");
}

#[test]
fn links_1() {
    let mut doc = Document::new();
    let mut svg_n = doc.create_element(EId::Svg);
    let mut use_n = doc.create_element(EId::Use);

    svg_n.set_id("svg1");

    doc.root().append(svg_n.clone());
    svg_n.append(use_n.clone());

    use_n.set_attribute((AId::Href, svg_n));

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" id=\"svg1\">
    <use xlink:href=\"#svg1\"/>
</svg>
");
}

#[test]
fn links_2() {
    let mut doc = Document::new();
    let mut svg_n = doc.create_element(EId::Svg);
    let mut lg_n = doc.create_element(EId::LinearGradient);
    let mut rect_n = doc.create_element(EId::Rect);

    lg_n.set_id("lg1");

    doc.root().append(svg_n.clone());
    svg_n.append(lg_n.clone());
    svg_n.append(rect_n.clone());

    rect_n.set_attribute((AId::Fill, lg_n));

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\">
    <linearGradient id=\"lg1\"/>
    <rect fill=\"url(#lg1)\"/>
</svg>
");
}

#[test]
fn attributes_types_1() {
    let mut doc = Document::new();
    let mut svg = doc.create_element(EId::Svg);

    doc.root().append(svg.clone());

    svg.set_attribute((AId::ViewBox, ViewBox::new(10.0, 20.0, 30.0, 40.0)));
    svg.set_attribute((AId::Version, "1.0"));
    svg.set_attribute((AId::Width, 1.5));
    svg.set_attribute((AId::Height, Length::new(1.5, LengthUnit::Percent)));
    svg.set_attribute((AId::Fill, Color::white()));
    svg.set_attribute((AId::Transform, Transform::new(2.0, 0.0, 0.0, 3.0, 20.0, 30.0)));
    svg.set_attribute((AId::StdDeviation, NumberList(vec![1.5, 2.5])));

    svg.set_attribute((AId::StrokeDasharray, LengthList(vec![
        Length::new(1.5, LengthUnit::Mm),
        Length::new(2.5, LengthUnit::Mm),
        Length::new(3.5, LengthUnit::Mm),
    ])));

    // TODO: add path

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;

    assert_eq!(doc.with_write_opt(&opt).to_string(),
        "<svg xmlns='http://www.w3.org/2000/svg' fill='#ffffff' height='1.5%' \
         stdDeviation='1.5 2.5' stroke-dasharray='1.5mm 2.5mm 3.5mm' \
         transform='matrix(2 0 0 3 20 30)' version='1.0' viewBox='10 20 30 40' \
         width='1.5'/>\n");
}

#[test]
fn comment_1() {
    let mut doc = Document::new();

    let comm = doc.create_node(NodeType::Comment, "comment");
    let svg = doc.create_element(EId::Svg);

    doc.root().append(comm);
    doc.root().append(svg);

    assert_eq!(doc.to_string(), "<!--comment-->\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n");
}

test_resave!(cdata_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script><![CDATA[
        text
    ]]></script>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script>text</script>
</svg>
");

test_resave!(cdata_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script><![CDATA[]]></script>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script/>
</svg>
");

test_resave!(cdata_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script><![CDATA[qwe]]>qwe<![CDATA[qwe]]></script>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script>qweqweqwe</script>
</svg>
");

test_resave!(cdata_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script><![CDATA[<text/>]]></script>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <script>&lt;text/&gt;</script>
</svg>
");

#[test]
fn indent_1() {
    // default indent is 4

    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\">
    <g>
        <rect/>
    </g>
</svg>
");
}

#[test]
fn indent_2() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Spaces(2);
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
  <g>
    <rect/>
  </g>
</svg>
");
}

#[test]
fn indent_3() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Spaces(0);
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
<g>
<rect/>
</g>
</svg>
");
}

#[test]
fn indent_4() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'><g><rect/></g></svg>");
}

#[test]
fn indent_5() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Tabs;
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
\t<g>
\t\t<rect/>
\t</g>
</svg>
");
}

#[test]
fn attrs_indent_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg' id='svg1' width='100' height='100'>
    <g fill='red' stroke='blue'>
        <rect id='rect1' stroke-width='2'/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.attributes_indent = Indent::Spaces(3);
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg
   xmlns='http://www.w3.org/2000/svg'
   id='svg1'
   height='100'
   width='100'>
    <g
       fill='#ff0000'
       stroke='#0000ff'>
        <rect
           id='rect1'
           stroke-width='2'/>
    </g>
</svg>
");
}

#[test]
fn single_quote_1() {
    let doc = Document::from_str("<svg xmlns='http://www.w3.org/2000/svg' id=\"svg1\"/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns='http://www.w3.org/2000/svg' id='svg1'/>");
}

test_resave!(escape_1,
"<svg xmlns='http://www.w3.org/2000/svg' unicode='ffl'/>",
"<svg xmlns='http://www.w3.org/2000/svg' unicode='&#x66;&#x66;&#x6c;'/>
");

// Do not escape already escaped.
test_resave!(escape_2,
"<svg xmlns='http://www.w3.org/2000/svg' unicode='&#x66;&#x66;&#x6c;'/>",
"<svg xmlns='http://www.w3.org/2000/svg' unicode='&#x66;&#x66;&#x6c;'/>
");

// Escape attribute values according to the current quote type.
#[test]
fn escape_3() {
    let doc = Document::from_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" font-family=\"'Noto Sans'\"/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;

    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns=\"http://www.w3.org/2000/svg\" font-family=\"'Noto Sans'\"/>");

    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns='http://www.w3.org/2000/svg' font-family='&apos;Noto Sans&apos;'/>");
}

// Escape attribute values according to the current quote type.
#[test]
fn escape_4() {
    let doc = Document::from_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" font-family='\"Noto Sans\"'/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;

    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns=\"http://www.w3.org/2000/svg\" font-family=\"&quot;Noto Sans&quot;\"/>");

    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns='http://www.w3.org/2000/svg' font-family='\"Noto Sans\"'/>");
}

#[test]
fn attrs_order_1() {
    let doc = Document::from_str(
        "<svg xmlns='http://www.w3.org/2000/svg' id='svg1' width='100' height='100' fill='#ff0000' stroke='#0000ff'/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;
    opt.use_single_quote = true;

    opt.attributes_order = AttributesOrder::AsIs;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
        "<svg xmlns='http://www.w3.org/2000/svg' id='svg1' width='100' height='100' fill='#ff0000' stroke='#0000ff'/>");

    opt.attributes_order = AttributesOrder::Alphabetical;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
        "<svg xmlns='http://www.w3.org/2000/svg' id='svg1' fill='#ff0000' height='100' stroke='#0000ff' width='100'/>");
}

#[test]
fn attrs_order_2() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient x1='1' gradientTransform='scale(2)' y1='1' gradientUnits='userSpaceOnUse' \
        spreadMethod='pad' x2='1' y2='1'/>
    <rect fill='#ff0000' height='5' y='5' x='5' width='5' stroke='#ff0000'/>

</svg>"
).unwrap();

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    opt.attributes_order = AttributesOrder::Specification;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient x1='1' y1='1' x2='1' y2='1' gradientUnits='userSpaceOnUse' \
        gradientTransform='matrix(2 0 0 2 0 0)' spreadMethod='pad'/>
    <rect fill='#ff0000' stroke='#ff0000' x='5' y='5' width='5' height='5'/>
</svg>
"
);
}

// attrs_order_3 with non-svg attr

test_resave!(namespaces_1,
"<svg xmlns='http://www.w3.org/2000/svg'/>",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

test_resave!(namespaces_2,
"<svg:svg xmlns:svg='http://www.w3.org/2000/svg'/>",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

test_resave!(namespaces_3,
"<svg:svg svg:x='0' xmlns:svg='http://www.w3.org/2000/svg'/>",
"<svg xmlns='http://www.w3.org/2000/svg' x='0'/>
");

// Non-SVG element.
test_resave!(namespaces_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <d:SVGTestCase xmlns:d='http://www.w3.org/2000/02/svg/testsuite/description/'>
        <rect/>
    </d:SVGTestCase>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

test_resave!(aspect_ratio_1,
"<svg xmlns='http://www.w3.org/2000/svg' preserveAspectRatio='defer none slice'/>",
"<svg xmlns='http://www.w3.org/2000/svg' preserveAspectRatio='defer none slice'/>
");

test_resave!(non_svg_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect my-attr='qwe'/>
    <random/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>
");
