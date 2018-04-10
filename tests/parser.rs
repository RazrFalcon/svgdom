// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate svgdom;
extern crate simplecss;

use svgdom::{
    AttributeId as AId,
    AttributeValue,
    Color,
    Document,
    ElementId as EId,
    TagNameRef,
    NodeType,
    ParseOptions,
    ToStringWithOptions,
    ValueId,
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
            assert_eq_text!(doc.to_string_with_opt(&write_options()), $out_text);
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
    let doc = Document::from_str("<?xml version='1.0' encoding='UTF-8' standalone='no'?><svg/>").unwrap();

    let child = doc.root().first_child().unwrap();
    assert_eq!(child.node_type(), NodeType::Declaration);
    // we store declaration only with double quotes
    assert_eq!(child.text(), "version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"");
    assert_eq!(doc.root().children().count(), 2);
}

#[test]
fn parse_comment_1() {
    let doc = Document::from_str("<svg/><!--comment-->").unwrap();

    let child = doc.root().children().nth(1).unwrap();
    assert_eq!(child.node_type(), NodeType::Comment);
    assert_eq!(child.text(), "comment");
    assert_eq!(doc.root().children().count(), 2);
}

#[test]
fn parse_text_1() {
    let doc = Document::from_str("<svg>text</svg>").unwrap();

    let child = doc.root().first_child().unwrap().first_child().unwrap();
    assert_eq!(child.node_type(), NodeType::Text);
    assert_eq!(child.text(), "text");
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
    assert_eq!(text_data_node.text(), "Some");
    assert_eq!(text_data_node.node_type(), NodeType::Text);

    let tspan_node = nodes.next().unwrap();
    assert_eq!(tspan_node.tag_name().as_ref(), TagNameRef::from(EId::Tspan));
    assert_eq!(tspan_node.node_type(), NodeType::Element);

    let text_data_node_2 = nodes.next().unwrap();
    assert_eq!(text_data_node_2.text(), "complex");
    assert_eq!(text_data_node_2.node_type(), NodeType::Text);

    let text_data_node_3 = nodes.next().unwrap();
    assert_eq!(text_data_node_3.text(), "text");
    assert_eq!(text_data_node_3.node_type(), NodeType::Text);
}

test_resave!(parse_css_1,
"<svg>
    <style type='text/css'>
        <![CDATA[
            .fil1 {fill:#00913f}
            .str1{stroke:#ffcc00;stroke-width:2}
            .str2  {stroke-linejoin:round;}
        ]]>
    </style>
    <g class='fil1'/>
    <g class='str1 str2'/>
</svg>
",
"<svg>
    <g fill='#00913f'/>
    <g stroke='#ffcc00' stroke-linejoin='round' stroke-width='2'/>
</svg>
");

// style can be set after usage
test_resave!(parse_css_2,
"<svg>
    <g class='fil1'/>
    <style type='text/css'>
        <![CDATA[ .fil1 {fill:#00913f} ]]>
    </style>
</svg>
",
"<svg>
    <g fill='#00913f'/>
</svg>
");

test_resave!(parse_css_4,
"<svg>
    <style type='text/css'>
    <![CDATA[
        rect {fill:red;}
    ]]>
    </style>
    <rect/>
    <rect/>
</svg>
",
"<svg>
    <rect fill='#ff0000'/>
    <rect fill='#ff0000'/>
</svg>
");

// empty data
test_resave!(parse_css_5,
"<svg>
    <style type='text/css'>
    </style>
</svg>
",
"<svg/>
");

// multiline comments and styles
test_resave!(parse_css_6,
"<svg>
    <style type='text/css'>
    <![CDATA[
        /*
         * Below are Cascading Style Sheet (CSS) definitions in use in this file,
         * which allow easily changing how elements are displayed.
         *
         */
        .circle
        {
          opacity:0;
          fill:#b9b9b9;
          fill-opacity:1;
        }
        /*
         * Comment
         */
    ]]>
    </style>
    <g class='circle'/>
</svg>",
"<svg>
    <g fill='#b9b9b9' fill-opacity='1' opacity='0'/>
</svg>
");

// links should be properly linked
test_resave!(parse_css_7,
"<svg>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:url(#lg1)}
    ]]>
    </style>
    <radialGradient id='lg1'/>
    <rect class='fil1'/>
</svg>",
"<svg>
    <radialGradient id='lg1'/>
    <rect fill='url(#lg1)'/>
</svg>
");

// order of styles ungrouping is important
test_resave!(parse_css_8,
"<svg>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:blue}
    ]]>
    </style>
    <g fill='red' style='fill:green' class='fil1'/>
</svg>",
"<svg>
    <g fill='#008000'/>
</svg>
");

// order of styles ungrouping is important
test_resave!(parse_css_9,
"<svg>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:blue}
    ]]>
    </style>
    <g fill='red' class='fil1'/>
</svg>",
"<svg>
    <g fill='#0000ff'/>
</svg>
");

// style can be set without CDATA block
test_resave!(parse_css_10,
"<svg>
    <style type='text/css'>
        .fil1 {fill:blue}
    </style>
    <g fill='red' class='fil1'/>
</svg>",
"<svg>
    <g fill='#0000ff'/>
</svg>
");

#[test]
fn parse_css_11() {
    let res = Document::from_str(
"<svg>
    <style type='text/css'><![CDATA[
        @import url('../some.css');
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
        "Unsupported token at 3:9");
}

test_resave!(parse_css_12,
"<svg>
    <style type='text/css'><![CDATA[
        #c { fill: red }
        ]]>
    </style>
    <g id='c'/>
</svg>",
"<svg>
    <g id='c' fill='#ff0000'/>
</svg>
");

#[test]
fn parse_css_13() {
    let res = Document::from_str(
"<svg>
    <style type='text/css'><![CDATA[
        :lang(en) { fill: green}
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
        "unsupported CSS at 3:9");
}

test_resave!(parse_css_14,
"<svg>
    <style type='text/css'><![CDATA[
        * { fill: red }
        ]]>
    </style>
    <g>
        <rect/>
    </g>
    <path/>
</svg>",
"<svg fill='#ff0000'>
    <g fill='#ff0000'>
        <rect fill='#ff0000'/>
    </g>
    <path fill='#ff0000'/>
</svg>
");

#[test]
fn parse_css_15() {
    let res = Document::from_str(
"<svg>
    <style type='text/css'><![CDATA[
        a > b { fill: green}
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
        "unsupported CSS at 3:10");
}

#[test]
fn parse_css_16() {
    let res = Document::from_str(
"<svg>
    <style type='text/css'><![CDATA[
        g rect { fill: green }
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
        "unsupported CSS at 3:10");
}

// empty style
test_resave!(parse_css_17,
"<svg>
    <style type='text/css'/>
    <g fill='#0000ff'/>
</svg>",
"<svg>
    <g fill='#0000ff'/>
</svg>
");

test_resave!(parse_css_18,
"<svg>
    <style type='text/css'>
        .fil1, .fil2 {fill:blue}
    </style>
    <g class='fil1'/>
    <g class='fil2'/>
</svg>",
"<svg>
    <g fill='#0000ff'/>
    <g fill='#0000ff'/>
</svg>
");

test_resave!(parse_css_19,
"<svg>
    <style type='text/css'>
    <![CDATA[
    ]]>
    </style>
</svg>",
"<svg/>
");

test_resave!(parse_css_20,
"<svg>
    <style type='text/css'>
        .cls-1,.cls-17{fill:red;}
        .cls-1{stroke:red;}
        .cls-17{stroke:black;}
    </style>
    <g class='cls-1'/>
    <g class='cls-17'/>
</svg>",
"<svg>
    <g fill='#ff0000' stroke='#ff0000'/>
    <g fill='#ff0000' stroke='#000000'/>
</svg>
");

test_resave!(parse_css_21,
"<svg>
    <style>#g1 { fill:red }</style>
    <style type='text/css'>#g1 { fill:blue }</style>
    <style type='blah'>#g1 { fill:red }</style>
    <g id='g1'/>
</svg>",
"<svg>
    <g id='g1' fill='#0000ff'/>
</svg>
");

// space before closing tag
test_resave!(parse_css_22,
"<svg>
<style type='text/css' >
<![CDATA[
rect {fill:red;}
]]>
</style>
<rect/>
</svg>
",
"<svg>
    <rect fill='#ff0000'/>
</svg>
");

// marker property
test_resave!(parse_css_23,
"<svg>
    <style type='text/css' >
        rect { marker: url(#marker1); }
    </style>
    <marker id='marker1'/>
    <rect/>
</svg>
",
"<svg>
    <marker id='marker1'/>
    <rect marker-end='url(#marker1)' marker-mid='url(#marker1)' marker-start='url(#marker1)'/>
</svg>
");

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
fn parse_iri_1() {
    let doc = Document::from_str(
"<svg>
    <radialGradient id='rg1'/>
    <rect fill='url(#rg1)'/>
</svg>").unwrap();

    let child = doc.root().first_child().unwrap();
    let rg = child.children().nth(0).unwrap();
    let rect = child.children().nth(1).unwrap();

    assert_eq!(rg.is_used(), true);
    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(), &AttributeValue::FuncLink(rg));
}

#[test]
fn parse_iri_2() {
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
    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(), &AttributeValue::FuncLink(rg));
}

#[test]
fn parse_iri_with_fallback_1() {
    let doc = Document::from_str(
"<svg>
    <rect fill='url(#lg1) none'/>
</svg>").unwrap();

    let child = doc.root().first_child().unwrap();
    let rect = child.children().nth(0).unwrap();

    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(),
               &AttributeValue::PredefValue(ValueId::None));
}

#[test]
fn parse_iri_with_fallback_2() {
    let doc = Document::from_str(
"<svg>
    <rect fill='url(#lg1) red'/>
</svg>").unwrap();

    let child = doc.root().first_child().unwrap();
    let rect = child.children().nth(0).unwrap();

    assert_eq!(rect.attributes().get_value(AId::Fill).unwrap(),
               &AttributeValue::Color(Color::new(255, 0, 0)));
}

#[test]
fn parse_iri_with_fallback_3() {
    // unsupported case

    let doc = Document::from_str(
"<svg>
    <radialGradient id='rg1'/>
    <rect fill='url(#rg1) none'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "valid FuncIRI(#rg1) with fallback value is not supported");
}

#[test]
fn parse_iri_with_fallback_4() {
    // unsupported case

    let doc = Document::from_str(
"<svg>
    <rect fill='url(#rg1) none'/>
    <radialGradient id='rg1'/>
</svg>");

    assert_eq!(doc.err().unwrap().to_string(),
               "valid FuncIRI(#rg1) with fallback value is not supported");
}

test_resave!(parse_filter_iri_1,
"<svg>
    <rect filter='url(#rg1)'/>
</svg>",
"<svg>
    <rect visibility='hidden'/>
</svg>
");

test_resave!(parse_filter_iri_2,
"<svg>
    <mask>
        <rect filter='url(#rg1)'/>
    </mask>
</svg>",
"<svg>
    <mask>
        <rect/>
    </mask>
</svg>
");

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

#[test]
fn parse_entity_5() {
    let doc = Document::from_str(
"<!DOCTYPE svg [
    <!ENTITY Viewport1 \"<rect/>\">
]>
<svg fill='&st1;'/>");
    assert_eq!(doc.err().unwrap().to_string(),
               "unsupported ENTITY data at 2:25");
}

#[test]
fn parse_entity_6() {
    let doc = Document::from_str(
"<!DOCTYPE svg [
    <!ENTITY Viewport1 \" \t\n<rect/>\">
]>
<svg fill='&st1;'/>");
    assert_eq!(doc.err().unwrap().to_string(),
               "unsupported ENTITY data at 2:25");
}

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
fn skip_comments_1() {
    let mut opt = ParseOptions::default();
    opt.parse_comments = false;
    let doc = Document::from_str_with_opt(
"<!--comment-->
<svg/>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg/>
");
}

#[test]
fn skip_declaration_1() {
    let mut opt = ParseOptions::default();
    opt.parse_declarations = false;
    let doc = Document::from_str_with_opt(
"<?xml version='1.0'?>
<svg/>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg/>
");
}

#[test]
fn skip_unknown_elements_1() {
    let mut opt = ParseOptions::default();
    opt.parse_unknown_elements = false;
    let doc = Document::from_str_with_opt(
"<svg>
    <qwe id='q'/>
    <rect/>
</svg>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg>
    <rect/>
</svg>
");
}

#[test]
fn skip_unknown_elements_2() {
    let mut opt = ParseOptions::default();
    opt.parse_unknown_elements = false;
    let doc = Document::from_str_with_opt(
"<svg>
    <qwe>
        <qwe>
            <rect/>
        </qwe>
    </qwe>
    <rect/>
</svg>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg>
    <rect/>
</svg>
");
}

#[test]
fn skip_unknown_elements_3() {
    let mut opt = ParseOptions::default();
    opt.parse_unknown_elements = false;
    let doc = Document::from_str_with_opt(
"<svg>
    <qwe>
        <rect/>
        <rect/>
        <rect/>
    </qwe>
    <rect/>
</svg>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg>
    <rect/>
</svg>
");
}

#[test]
fn skip_unknown_elements_4() {
    let mut opt = ParseOptions::default();
    opt.parse_unknown_elements = false;
    let doc = Document::from_str_with_opt(
"<svg>
    <qwe>
    </qwe>
    <rect/>
</svg>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg>
    <rect/>
</svg>
");
}

#[test]
fn skip_unknown_attributes_1() {
    let mut opt = ParseOptions::default();
    opt.parse_unknown_attributes = false;
    let doc = Document::from_str_with_opt(
"<svg fill='#ff0000' test='1' qwe='zzz' xmlns='http://www.w3.org/2000/svg' \
xmlns:xlink='http://www.w3.org/1999/xlink'/>", &opt).unwrap();

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg fill='#ff0000' xmlns:xlink='http://www.w3.org/1999/xlink' \
xmlns='http://www.w3.org/2000/svg'/>
");
}

#[test]
fn parse_px_unit_on_1() {
    let mut opt = ParseOptions::default();
    opt.parse_px_unit = true;
    let doc = Document::from_str_with_opt("<svg x='10px'/>", &opt).unwrap();
    assert_eq_text!(doc.to_string_with_opt(&write_options()), "<svg x='10px'/>\n");
}

#[test]
fn parse_px_unit_off_1() {
    let mut opt = ParseOptions::default();
    opt.parse_px_unit = false;
    let doc = Document::from_str_with_opt("<svg x='10px'/>", &opt).unwrap();
    assert_eq_text!(doc.to_string_with_opt(&write_options()), "<svg x='10'/>\n");
}

#[test]
fn parse_px_unit_off_2() {
    let mut opt = ParseOptions::default();
    opt.parse_px_unit = false;
    let doc = Document::from_str_with_opt("<svg stroke-dasharray='10px 20px'/>", &opt).unwrap();
    assert_eq_text!(doc.to_string_with_opt(&write_options()),
                    "<svg stroke-dasharray='10 20'/>\n");
}

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

    assert_eq_text!(doc.to_string_with_opt(&write_options()),
"<svg>
    <g class='fil3' fill='#0000ff'/>
    <g class='fil4 fil5' fill='#0000ff' stroke='#0000ff'/>
</svg>
");
}

// TODO: this
// p { font-family: "Font 1", "Font 2", Georgia, Times, serif; }

#[test]
fn text_content_1() {
    let doc = Document::from_str(
"<svg>
    <text>
        A <tspan>
            <tspan>
                link
                inside tspan
            </tspan> for testing
        </tspan>
    </text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "A link inside tspan for testing");
}

#[test]
fn text_content_2() {
    let doc = Document::from_str(
"<svg>
    <text>
        <tspan>Text1</tspan>
        <tspan>Text2</tspan>
        <tspan>Text3</tspan>
    </text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "Text1 Text2 Text3");
}

#[test]
fn text_content_3() {
    let doc = Document::from_str(
"<svg>
    <text>
      Not

      <tspan>
        all characters

        <tspan>
          in

          <tspan>
            the
          </tspan>
        </tspan>

        <tspan>
          text
        </tspan>

        have a
      </tspan>

      <tspan>
        specified
      </tspan>

      rotation
    </text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "Not all characters in the text have a specified rotation");
}

#[test]
fn text_content_4() {
    let doc = Document::from_str(
"<svg>
    <text>
        <tspan xml:space='preserve'>
            Text
        </tspan>
        Text
    </text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "             Text         Text");
}
