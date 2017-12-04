// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate svgdom;

use svgdom::{
    AttributeId as AId,
    Document,
    ElementId as EId,
    Indent,
    NodeType,
    WriteOptions,
    ToStringWithOptions,
};
use svgdom::types::{
    Color,
    Length,
    LengthUnit,
    Transform,
};

macro_rules! test_resave {
    ($name:ident, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_str($in_text).unwrap();

            let mut opt = WriteOptions::default();
            opt.use_single_quote = true;

            assert_eq_text!(doc.to_string_with_opt(&opt), $out_text);
        }
    )
}

#[test]
fn empty_doc_1() {
    assert_eq_text!(Document::new().to_string(), String::new());
}

#[test]
fn single_node_1() {
    let mut doc = Document::new();
    let n = doc.create_element(EId::Svg);

    doc.append(&n);

    assert_eq_text!(doc.to_string(), "<svg/>\n");
}

#[test]
fn child_node_1() {
    let mut doc = Document::new();
    let mut svg = doc.create_element(EId::Svg);
    let defs = doc.create_element(EId::Defs);

    doc.append(&svg);
    svg.append(&defs);

    assert_eq_text!(doc.to_string(),
"<svg>
    <defs/>
</svg>
");
}

#[test]
fn child_nodes_1() {
    let mut doc = Document::new();
    let svg = doc.create_element(EId::Svg);
    doc.append(&svg);

    let mut parent = svg;
    for n in 1..5 {
        let mut r = doc.create_element(EId::Rect);
        r.set_id(n.to_string());
        parent.append(&r);

        parent = r;
    }

    assert_eq_text!(doc.to_string(),
"<svg>
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

    doc.append(&svg_n);
    svg_n.append(&use_n);

    use_n.set_attribute((AId::XlinkHref, svg_n));

    assert_eq_text!(doc.to_string(),
"<svg id=\"svg1\">
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

    doc.append(&svg_n);
    svg_n.append(&lg_n);
    svg_n.append(&rect_n);

    rect_n.set_attribute((AId::Fill, lg_n));

    assert_eq_text!(doc.to_string(),
"<svg>
    <linearGradient id=\"lg1\"/>
    <rect fill=\"url(#lg1)\"/>
</svg>
");
}

#[test]
fn attributes_types_1() {
    let mut doc = Document::new();
    let mut svg = doc.create_element(EId::Svg);

    doc.append(&svg);

    svg.set_attribute((AId::Version, "1.0"));
    svg.set_attribute((AId::Width, 1.5));
    svg.set_attribute((AId::Height, Length::new(1.5, LengthUnit::Percent)));
    svg.set_attribute((AId::Fill, Color::new(255, 255, 255)));
    svg.set_attribute((AId::Transform, Transform::new(2.0, 0.0, 0.0, 3.0, 20.0, 30.0)));
    svg.set_attribute((AId::StdDeviation, vec![1.5, 2.5, 3.5]));

    let mut len_list = Vec::new();
    len_list.push(Length::new(1.5, LengthUnit::Mm));
    len_list.push(Length::new(2.5, LengthUnit::Mm));
    len_list.push(Length::new(3.5, LengthUnit::Mm));
    svg.set_attribute((AId::StrokeDasharray, len_list));

    // TODO: add path

    assert_eq_text!(doc.to_string(),
        "<svg fill=\"#ffffff\" height=\"1.5%\" \
         stdDeviation=\"1.5 2.5 3.5\" stroke-dasharray=\"1.5mm 2.5mm 3.5mm\" \
         transform=\"matrix(2 0 0 3 20 30)\" version=\"1.0\" width=\"1.5\"/>\n");
}

#[test]
fn declaration_1() {
    let mut doc = Document::new();

    let dec = doc.create_node(NodeType::Declaration,
        "version=\"1.0\" encoding=\"UTF-8\"");
    let svg = doc.create_element(EId::Svg);

    doc.append(&dec);
    doc.append(&svg);

    assert_eq_text!(doc.to_string(), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg/>\n");
}

#[test]
fn comment_1() {
    let mut doc = Document::new();

    let comm = doc.create_node(NodeType::Comment, "comment");
    let svg = doc.create_element(EId::Svg);

    doc.append(&comm);
    doc.append(&svg);

    assert_eq_text!(doc.to_string(), "<!--comment-->\n<svg/>\n");
}

// Manually created text.
#[test]
fn text_1() {
    let mut doc = Document::new();

    let mut svg = doc.create_element(EId::Svg);
    let text = doc.create_node(NodeType::Text, "text");

    doc.append(&svg);
    svg.append(&text);

    assert_eq_text!(doc.to_string(),
"<svg>text</svg>
");
}

// Text inside non-svg element.
test_resave!(text_2,
"<svg>
    <p>
        text
    </p>
</svg>
",
"<svg>
    <p>text</p>
</svg>
");

// Text inside svg element.
test_resave!(text_3,
"<svg>
    <text>
        text
    </text>
</svg>
",
"<svg>
    <text>text</text>
</svg>
");

// Multiline text.
test_resave!(text_4,
"<svg>
    <text>
        Line 1
        Line 2
        Line 3
    </text>
</svg>
",
"<svg>
    <text>Line 1 Line 2 Line 3</text>
</svg>
");

// Multiline text with 'preserve'.
test_resave!(text_5,
"<svg>
    <text xml:space='preserve'>
        Line 1
        Line 2
        Line 3
    </text>
</svg>
",
"<svg>
    <text xml:space='preserve'>         Line 1         Line 2         Line 3     </text>
</svg>
");

// Test trimming.
// Details: https://www.w3.org/TR/SVG11/text.html#WhiteSpace
test_resave!(text_6,
"<svg>
    <text></text>
    <text> </text>
    <text>  </text>
    <text> \t \n \r </text>
    <text> \t  text \t  text  t \t\n  </text>
    <text xml:space='preserve'> \t \n text \t  text  t \t \r\n\r\n</text>
</svg>
",
"<svg>
    <text/>
    <text></text>
    <text></text>
    <text></text>
    <text>text text t</text>
    <text xml:space='preserve'>     text    text  t     </text>
</svg>
");

// Escape.
test_resave!(text_7,
"<svg>
    <text>&amp;&lt;&gt;</text>
    <nontext>&amp;&lt;&gt;</nontext>
</svg>
",
"<svg>
    <text>&amp;&lt;&gt;</text>
    <nontext>&amp;&lt;&gt;</nontext>
</svg>
");

test_resave!(text_8,
"<svg>
    <text>Text</text>
    <rect/>
</svg>
",
"<svg>
    <text>Text</text>
    <rect/>
</svg>
");

// Text with children elements.
// Spaces will be trimmed, but not all.
test_resave!(text_tspan_1,
"<svg>
    <text>
      Some \t <tspan>  complex  </tspan>  text \t
    </text>
</svg>
",
"<svg>
    <text>Some <tspan>complex </tspan>text</text>
</svg>
");

// Text with tspan but without spaces.
test_resave!(text_tspan_2,
"<svg>
    <text><tspan>Text</tspan></text>
</svg>
",
"<svg>
    <text>
        <tspan>Text</tspan>
    </text>
</svg>
");

// Text with tspan with new lines.
test_resave!(text_tspan_3,
"<svg>
    <text>
        <tspan>Text</tspan>
        <tspan>Text</tspan>
        <tspan>Text</tspan>
    </text>
</svg>
",
"<svg>
    <text><tspan>Text</tspan><tspan>Text</tspan><tspan>Text</tspan></text>
</svg>
");

// Text with spaces inside a tspan.
test_resave!(text_tspan_4,
"<svg>
    <text>Some<tspan> long </tspan>text</text>
</svg>
",
"<svg>
    <text>Some<tspan> long </tspan>text</text>
</svg>
");

// Text with spaces outside a tspan.
test_resave!(text_tspan_5,
"<svg>
    <text>Some <tspan>long</tspan> text</text>
</svg>
",
"<svg>
    <text>Some <tspan>long</tspan> text</text>
</svg>
");

// Nested tspan.
test_resave!(text_tspan_6,
"<svg>
    <text>  Some  <tspan>  not  <tspan>  very  </tspan>  long  </tspan>  text  </text>
</svg>
",
"<svg>
    <text>Some <tspan>not <tspan>very </tspan>long </tspan>text</text>
</svg>
");

// Empty tspan.
test_resave!(text_tspan_7,
"<svg>
    <text><tspan><tspan></tspan></tspan></text>
    <text> <tspan> <tspan> </tspan> </tspan> </text>
</svg>
",
"<svg>
    <text>
        <tspan>
            <tspan/>
        </tspan>
    </text>
    <text><tspan><tspan></tspan></tspan></text>
</svg>
");

test_resave!(text_tspan_8,
"<svg>
    <text>
        <tspan>Te</tspan><tspan>x</tspan>t
    </text>
</svg>",
"<svg>
    <text><tspan>Te</tspan><tspan>x</tspan>t</text>
</svg>
");

// Test xml:space.
test_resave!(text_space_preserve_1,
"<svg>
    <text xml:space='preserve'> Text
    </text>
</svg>
",
"<svg>
    <text xml:space='preserve'> Text     </text>
</svg>
");

// Test xml:space inheritance.
test_resave!(text_space_preserve_2,
"<svg xml:space='preserve'>
    <text> Text
    </text>
</svg>
",
"<svg xml:space='preserve'>
    <text> Text     </text>
</svg>
");

// Test mixed xml:space.
test_resave!(text_space_preserve_3,
"<svg xml:space='preserve'>
    <text>
    Text
    <tspan xml:space='default'>
    Text
    </tspan>
    Text
    </text>
</svg>
",
"<svg xml:space='preserve'>
    <text>     Text     <tspan xml:space='default'> Text </tspan>     Text     </text>
</svg>
");

test_resave!(text_space_preserve_4,
"<svg>
    <g>
        <text> Text <tspan xml:space='preserve'> Text </tspan> Text </text>
    </g>
</svg>
",
"<svg>
    <g>
        <text>Text <tspan xml:space='preserve'> Text </tspan> Text</text>
    </g>
</svg>
");

test_resave!(text_space_preserve_5,
"<svg>
    <text>
        Text<tspan xml:space='preserve'> Text </tspan>Text
    </text>
</svg>
",
"<svg>
    <text>Text<tspan xml:space='preserve'> Text </tspan>Text</text>
</svg>
");

test_resave!(cdata_1,
"<svg>
    <script><![CDATA[
        js code
    ]]></script>
</svg>
",
"<svg>
    <script>
    <![CDATA[
        js code
    ]]>
    </script>
</svg>
");

test_resave!(cdata_2,
"<svg>
    <script><![CDATA[]]></script>
</svg>
",
"<svg>
    <script>
    <![CDATA[]]>
    </script>
</svg>
");

test_resave!(cdata_3,
"<svg>
    <script><![CDATA[qwe]]></script>
</svg>
",
"<svg>
    <script>
    <![CDATA[qwe]]>
    </script>
</svg>
");

test_resave!(cdata_4,
"<svg>
    <script><![CDATA[qwe]]><![CDATA[qwe]]><![CDATA[qwe]]></script>
</svg>
",
"<svg>
    <script>
    <![CDATA[qwe]]>
    <![CDATA[qwe]]>
    <![CDATA[qwe]]>
    </script>
</svg>
");

#[test]
fn indent_1() {
    // default indent is 4

    let doc = Document::from_str(
"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    assert_eq_text!(doc.to_string(),
"<svg>
    <g>
        <rect/>
    </g>
</svg>
");
}

#[test]
fn indent_2() {
    let doc = Document::from_str(
"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Spaces(2);
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
  <g>
    <rect/>
  </g>
</svg>
");
}

#[test]
fn indent_3() {
    let doc = Document::from_str(
"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Spaces(0);
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
<g>
<rect/>
</g>
</svg>
");
}

#[test]
fn indent_4() {
    let doc = Document::from_str(
"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg><g><rect/></g></svg>");
}

#[test]
fn indent_5() {
    let doc = Document::from_str(
"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::Tabs;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
\t<g>
\t\t<rect/>
\t</g>
</svg>
");
}

#[test]
fn attrs_indent_1() {
    let doc = Document::from_str(
"<svg id='svg1' width='100' height='100'>
    <g fill='red' stroke='blue' custom='qwe'>
        <rect id='rect1' stroke-width='2'/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.attributes_indent = Indent::Spaces(3);
    opt.use_single_quote = true;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg
   id='svg1'
   height='100'
   width='100'>
    <g
       fill='#ff0000'
       stroke='#0000ff'
       custom='qwe'>
        <rect
           id='rect1'
           stroke-width='2'/>
    </g>
</svg>
");
}


#[test]
fn single_quote_1() {
    let doc = Document::from_str(
"<svg id=\"svg1\"/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = Indent::None;
    opt.use_single_quote = true;
    assert_eq_text!(doc.to_string_with_opt(&opt), "<svg id='svg1'/>");
}
