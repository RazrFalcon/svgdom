// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use] extern crate pretty_assertions;

extern crate svgdom;

use std::fmt;

use svgdom::{
    Document,
    ElementId as EId,
    NodeType,
    WriteOptions,
    WriteBuffer,
};

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

            let mut opt = WriteOptions::default();
            opt.use_single_quote = true;

            assert_eq!(TStr($out_text), TStr(&doc.with_write_opt(&opt).to_string()));
        }
    )
}

#[test]
fn text_content_1() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        Text
        <tspan xml:space='preserve'>  Text  </tspan>
        Text
    </text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "Text   Text  Text");
}

#[test]
fn text_content_5() {
    let doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&#x20;&apos;Text&apos;&#x20;</text>
</svg>
").unwrap();

    let text: String = doc.root().descendants().map(|n| n.text().to_owned()).collect();
    assert_eq!(text, "'Text'");
}

// Manually created text.
#[test]
fn text_1() {
    let mut doc = Document::new();

    let mut svg = doc.create_element(EId::Svg);
    let text = doc.create_node(NodeType::Text, "text");

    doc.root().append(svg.clone());
    svg.append(text.clone());

    assert_eq!(doc.to_string(),
"<svg xmlns=\"http://www.w3.org/2000/svg\">text</svg>
");
}

// Text inside svg element.
test_resave!(text_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        text
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>text</text>
</svg>
");

// Multiline text.
test_resave!(text_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        Line 1
        Line 2
        Line 3
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Line 1 Line 2 Line 3</text>
</svg>
");

// Multiline text with 'preserve'.
test_resave!(text_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'>
        Line 1
        Line 2
        Line 3
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'>         Line 1         Line 2         Line 3     </text>
</svg>
");

// Test trimming.
// Details: https://www.w3.org/TR/SVG11/text.html#WhiteSpace
test_resave!(text_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text></text>
    <text> </text>
    <text>  </text>
    <text> \t \n \r </text>
    <text> \t  text \t  text  t \t\n  </text>
    <text xml:space='preserve'> \t \n text \t  text  t \t \r\n\r\n</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text/>
    <text/>
    <text/>
    <text/>
    <text>text text t</text>
    <text xml:space='preserve'>     text    text  t     </text>
</svg>
");

// Escape.
test_resave!(text_7,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&amp;&lt;&gt;</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&amp;&lt;&gt;</text>
</svg>
");

test_resave!(text_8,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Text</text>
    <rect/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Text</text>
    <rect/>
</svg>
");

test_resave!(text_9,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&#x20;&amp;Text&amp;&#x20;</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&amp;Text&amp;</text>
</svg>
");

test_resave!(text_10,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&#x20;&amp;&#64;&#x40;&amp;&#x20;</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>&amp;@@&amp;</text>
</svg>
");

// Text with children elements.
// Spaces will be trimmed, but not all.
test_resave!(text_tspan_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
      Some \t <tspan>  complex  </tspan>  text \t
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some <tspan>complex</tspan> text</text>
</svg>
");

// Text with tspan but without spaces.
test_resave!(text_tspan_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan>Text</tspan></text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan>Text</tspan></text>
</svg>
");

// Text with tspan with new lines.
test_resave!(text_tspan_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        <tspan>Text</tspan>
        <tspan>Text</tspan>
        <tspan>Text</tspan>
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan>Text</tspan> <tspan>Text</tspan> <tspan>Text</tspan></text>
</svg>
");

// Text with spaces inside a tspan.
test_resave!(text_tspan_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some<tspan> long </tspan>text</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some<tspan> long </tspan>text</text>
</svg>
");

// Text with spaces outside a tspan.
test_resave!(text_tspan_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some <tspan>long</tspan> text</text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some <tspan>long</tspan> text</text>
</svg>
");

// Nested tspan.
test_resave!(text_tspan_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>  Some  <tspan>  not  <tspan>  very  </tspan>  long  </tspan>  text  </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Some <tspan>not <tspan>very</tspan> long</tspan> text</text>
</svg>
");

// Empty tspan.
test_resave!(text_tspan_7,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan><tspan></tspan></tspan></text>
    <text> <tspan> <tspan> </tspan> </tspan> </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan><tspan/></tspan></text>
    <text><tspan><tspan/></tspan></text>
</svg>
");

test_resave!(text_tspan_8,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        <tspan>Te</tspan><tspan>x</tspan>t
    </text>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text><tspan>Te</tspan><tspan>x</tspan>t</text>
</svg>
");

test_resave!(text_tspan_9,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        text <tspan>
            <tspan>
                text
            </tspan> text
        </tspan>
    </text>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>text <tspan><tspan>text</tspan> text</tspan></text>
</svg>
");

test_resave!(text_tspan_10,
"<svg xmlns='http://www.w3.org/2000/svg'>
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
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Not <tspan>all characters <tspan>in <tspan>the</tspan></tspan> <tspan>text</tspan> \
        have a</tspan> <tspan>specified</tspan> rotation</text>
</svg>
");

// Test xml:space.
test_resave!(text_space_preserve_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'> Text
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'> Text     </text>
</svg>
");

// Test xml:space inheritance.
test_resave!(text_space_preserve_2,
"<svg xmlns='http://www.w3.org/2000/svg' xml:space='preserve'>
    <text> Text
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg' xml:space='preserve'>
    <text> Text     </text>
</svg>
");

// Test mixed xml:space.
test_resave!(text_space_preserve_3,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'>  Text  <tspan xml:space='default'>  Text  </tspan>  Text  </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'>  Text  <tspan xml:space='default'>Text </tspan>  Text  </text>
</svg>
");

test_resave!(text_space_preserve_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        Text
        <tspan xml:space='preserve'>  Text  </tspan>
        Text
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Text <tspan xml:space='preserve'>  Text  </tspan>Text</text>
</svg>
");

test_resave!(text_space_preserve_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>
        Text<tspan xml:space='preserve'>  Text  </tspan>Text
    </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text>Text<tspan xml:space='preserve'>  Text  </tspan>Text</text>
</svg>
");

test_resave!(text_space_preserve_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'><tspan>Text</tspan></text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text xml:space='preserve'><tspan>Text</tspan></text>
</svg>
");

// Test xml:space propagation
test_resave!(text_space_preserve_7,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text id='text1' xml:space='preserve'>  Text  </text>
    <text id='text2'>  Text  </text>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <text id='text1' xml:space='preserve'>  Text  </text>
    <text id='text2'>Text</text>
</svg>
");
