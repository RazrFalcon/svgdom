// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate svgdom;

use svgdom::{Document, WriteOptions, WriteToString, NodeType};
use svgdom::AttributeId as AId;
use svgdom::ElementId as EId;
use svgdom::types::{Transform, Length, LengthUnit, Color};

// TODO: rename to writer

macro_rules! assert_eq_text {
    ($left:expr, $right:expr) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!("assertion failed: `(left == right)` \
                           \nleft:  `{}`\nright: `{}`",
                           left_val, right_val)
                }
            }
        }
    })
}

macro_rules! test_resave {
    ($name:ident, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_data($in_text).unwrap();
            assert_eq_text!(doc.to_string(), $out_text);
        }
    )
}

#[test]
fn empty_doc_1() {
    assert_eq!(Document::new().to_string(), String::new());
}

#[test]
fn single_node_1() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    doc.append(&n);

    assert_eq!(doc.to_string(), "<svg/>\n");
}

#[test]
fn child_node_1() {
    let doc = Document::new();
    let svg = doc.create_element(EId::Svg);
    let defs = doc.create_element(EId::Defs);

    doc.append(&svg);
    svg.append(&defs);

    assert_eq!(doc.to_string(),
"<svg>
    <defs/>
</svg>
");
}

#[test]
fn child_nodes_1() {
    let doc = Document::new();
    let svg = doc.create_element(EId::Svg);
    doc.append(&svg);

    let mut parent = svg;
    for n in 1..5 {
        let r = doc.create_element(EId::Rect);
        r.set_id(n.to_string());
        parent.append(&r);

        parent = r;
    }

    assert_eq!(doc.to_string(),
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
    let doc = Document::new();
    let svg_n = doc.create_element(EId::Svg);
    let use_n = doc.create_element(EId::Use);

    svg_n.set_id("svg1");

    doc.append(&svg_n);
    svg_n.append(&use_n);

    use_n.set_link_attribute(AId::XlinkHref, svg_n).unwrap();

    assert_eq!(doc.to_string(),
"<svg id=\"svg1\">
    <use xlink:href=\"#svg1\"/>
</svg>
");
}

#[test]
fn links_2() {
    let doc = Document::new();
    let svg_n = doc.create_element(EId::Svg);
    let lg_n = doc.create_element(EId::LinearGradient);
    let rect_n = doc.create_element(EId::Rect);

    lg_n.set_id("lg1");

    doc.append(&svg_n);
    svg_n.append(&lg_n);
    svg_n.append(&rect_n);

    rect_n.set_link_attribute(AId::Fill, lg_n).unwrap();

    assert_eq!(doc.to_string(),
"<svg>
    <linearGradient id=\"lg1\"/>
    <rect fill=\"url(#lg1)\"/>
</svg>
");
}

#[test]
fn attributes_types_1() {
    let doc = Document::new();
    let svg = doc.create_element(EId::Svg);

    doc.append(&svg);

    svg.set_attribute(AId::Version, "1.0");
    svg.set_attribute(AId::Width, 1.5);
    svg.set_attribute(AId::Height, Length::new(1.5, LengthUnit::Percent));
    svg.set_attribute(AId::Fill, Color::new(255, 255, 255));
    svg.set_attribute(AId::Transform, Transform::new(2.0, 0.0, 0.0, 3.0, 20.0, 30.0));
    svg.set_attribute(AId::StdDeviation, vec![1.5, 2.5, 3.5]);

    let mut len_list = Vec::new();
    len_list.push(Length::new(1.5, LengthUnit::Mm));
    len_list.push(Length::new(2.5, LengthUnit::Mm));
    len_list.push(Length::new(3.5, LengthUnit::Mm));
    svg.set_attribute(AId::StrokeDasharray, len_list);

    // TODO: add path

    assert_eq!(doc.to_string(),
        "<svg fill=\"#ffffff\" height=\"1.5%\" \
         stdDeviation=\"1.5 2.5 3.5\" stroke-dasharray=\"1.5mm 2.5mm 3.5mm\" \
         transform=\"matrix(2 0 0 3 20 30)\" version=\"1.0\" width=\"1.5\"/>\n");
}

#[test]
fn declaration_1() {
    let doc = Document::new();

    let dec = doc.create_node(NodeType::Declaration,
        "version=\"1.0\" encoding=\"UTF-8\"");
    let svg = doc.create_element(EId::Svg);

    doc.append(&dec);
    doc.append(&svg);

    assert_eq!(doc.to_string(), "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg/>\n");
}

#[test]
fn comment_1() {
    let doc = Document::new();

    let comm = doc.create_node(NodeType::Comment, "comment");
    let svg = doc.create_element(EId::Svg);

    doc.append(&comm);
    doc.append(&svg);

    assert_eq!(doc.to_string(), "<!--comment-->\n<svg/>\n");
}

#[test]
fn text_1() {
    let doc = Document::new();

    let svg = doc.create_element(EId::Svg);
    let text = doc.create_node(NodeType::Text, "text");

    doc.append(&svg);
    svg.append(&text);

    assert_eq!(doc.to_string(),
"<svg>
    text
</svg>
");
}

test_resave!(text_2,
b"<svg>
    <p>
        text
    </p>
</svg>
",
"<svg>
    <p>
        text
    </p>
</svg>
");

// 'text' element has different behavior
test_resave!(text_3,
b"<svg>
    <text>
        text
    </text>
</svg>
",
"<svg>
    <text>
        text
    </text>
</svg>
");

test_resave!(text_4,
b"<svg>
    <p>
        text</p>
</svg>
",
"<svg>
    <p>
        text
    </p>
</svg>
");

test_resave!(text_multiline_1,
b"<svg>
    <p>
        Line 1
        Line 2
        Line 3
    </p>
</svg>
",
"<svg>
    <p>
        Line 1
        Line 2
        Line 3
    </p>
</svg>
");

// TODO: this
// #[test]
// fn text_multiline_2() {
//     let doc = parse_svg(
// b"<svg>
// <p>
// Line 1
// Line 2
// Line 3
// </p>
// </svg>
// ").unwrap();

//     println!("{}", dom_to_string(&doc));

//     assert_eq!(dom_to_string(&doc),
// "<svg>
//     <p>
//         Line 1
//         Line 2
//         Line 3
//     </p>
// </svg>
// ");
// }

test_resave!(text_mixed_indent,
b"<svg>
  <g>
      <p>
        text
      </p>
  </g>
</svg>
",
"<svg>
    <g>
        <p>
            text
        </p>
    </g>
</svg>
");

test_resave!(text_tspan_1,
b"<svg>
    <text>
      Some  <tspan> complex </tspan>  text \t
    </text>
</svg>
",
"<svg>
    <text>
        Some  <tspan> complex </tspan>  text
    </text>
</svg>
");

test_resave!(text_tspan_3,
b"<svg>
    <text>
        <tspan>Text</tspan>
    </text>
</svg>
",
"<svg>
    <text>
        <tspan>Text</tspan>
    </text>
</svg>
");

test_resave!(text_space_preserve,
b"<svg>
    <text xml:space='preserve'> Text
    </text>
</svg>
",
"<svg>
    <text xml:space=\"preserve\"> Text
    </text>
</svg>
");

#[test]
fn indent_1() {
    // default indent is 4

    let doc = Document::from_data(
b"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    assert_eq!(doc.to_string(),
"<svg>
    <g>
        <rect/>
    </g>
</svg>
");
}

#[test]
fn indent_2() {
    let doc = Document::from_data(
b"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = 2;
    assert_eq!(doc.to_string_with_opt(&opt),
"<svg>
  <g>
    <rect/>
  </g>
</svg>
");
}

#[test]
fn indent_3() {
    let doc = Document::from_data(
b"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = 0;
    assert_eq!(doc.to_string_with_opt(&opt),
"<svg>
<g>
<rect/>
</g>
</svg>
");
}

#[test]
fn indent_4() {
    let doc = Document::from_data(
b"<svg>
    <g>
        <rect/>
    </g>
</svg>
").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = -1;
    assert_eq!(doc.to_string_with_opt(&opt),
"<svg><g><rect/></g></svg>");
}

#[test]
fn single_quote_1() {
    let doc = Document::from_data(
b"<svg id=\"svg1\"/>").unwrap();

    let mut opt = WriteOptions::default();
    opt.indent = -1;
    opt.use_single_quote = true;
    assert_eq!(doc.to_string_with_opt(&opt), "<svg id='svg1'/>");
}
