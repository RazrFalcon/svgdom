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
    Document,
    WriteOptions,
    WriteBuffer,
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

test_resave!(parse_css_1,
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#00913f'/>
    <g stroke='#ffcc00' stroke-linejoin='round' stroke-width='2'/>
</svg>
");

// style can be set after usage
test_resave!(parse_css_2,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g class='fil1'/>
    <style type='text/css'>
        <![CDATA[ .fil1 {fill:#00913f} ]]>
    </style>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#00913f'/>
</svg>
");

test_resave!(parse_css_4,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    <![CDATA[
        rect {fill:red;}
    ]]>
    </style>
    <rect/>
    <rect/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect fill='#ff0000'/>
    <rect fill='#ff0000'/>
</svg>
");

// empty data
test_resave!(parse_css_5,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    </style>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

// multiline comments and styles
test_resave!(parse_css_6,
"<svg xmlns='http://www.w3.org/2000/svg'>
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
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#b9b9b9' fill-opacity='1' opacity='0'/>
</svg>
");

// links should be properly linked
test_resave!(parse_css_7,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:url(#lg1)}
    ]]>
    </style>
    <radialGradient id='lg1'/>
    <rect class='fil1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <radialGradient id='lg1'/>
    <rect fill='url(#lg1)'/>
</svg>
");

// order of styles ungrouping is important
test_resave!(parse_css_8,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:blue}
    ]]>
    </style>
    <g fill='red' style='fill:green' class='fil1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#008000'/>
</svg>
");

// order of styles ungrouping is important
test_resave!(parse_css_9,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    <![CDATA[
        .fil1 {fill:blue}
    ]]>
    </style>
    <g fill='red' class='fil1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#0000ff'/>
</svg>
");

// style can be set without CDATA block
test_resave!(parse_css_10,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
        .fil1 {fill:blue}
    </style>
    <g fill='red' class='fil1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#0000ff'/>
</svg>
");

#[test]
fn parse_css_11() {
    let res = Document::from_str(
        "\
<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        @import url('../some.css');
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
               "unsupported CSS at 2:37");
}

test_resave!(parse_css_12,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        #c { fill: red }
        ]]>
    </style>
    <g id='c'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='c' fill='#ff0000'/>
</svg>
");

#[test]
fn parse_css_13() {
    let res = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        :lang(en) { fill: green}
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
               "unsupported CSS at 2:37");
}

test_resave!(parse_css_14,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        * { fill: red }
        ]]>
    </style>
    <g>
        <rect/>
    </g>
    <path/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg' fill='#ff0000'>
    <g fill='#ff0000'>
        <rect fill='#ff0000'/>
    </g>
    <path fill='#ff0000'/>
</svg>
");

#[test]
fn parse_css_15() {
    let res = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        a > b { fill: green}
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
               "unsupported CSS at 2:37");
}

#[test]
fn parse_css_16() {
    let res = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'><![CDATA[
        g rect { fill: green }
        ]]>
    </style>
</svg>");

    assert_eq!(res.err().unwrap().to_string(),
               "unsupported CSS at 2:37");
}

// empty style
test_resave!(parse_css_17,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'/>
    <g fill='#0000ff'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#0000ff'/>
</svg>
");

test_resave!(parse_css_18,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
        .fil1, .fil2 {fill:blue}
    </style>
    <g class='fil1'/>
    <g class='fil2'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#0000ff'/>
    <g fill='#0000ff'/>
</svg>
");

test_resave!(parse_css_19,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
    <![CDATA[
    ]]>
    </style>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'/>
");

test_resave!(parse_css_20,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
        .cls-1,.cls-17{fill:red;}
        .cls-1{stroke:red;}
        .cls-17{stroke:black;}
    </style>
    <g class='cls-1'/>
    <g class='cls-17'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#ff0000' stroke='#ff0000'/>
    <g fill='#ff0000' stroke='#000000'/>
</svg>
");

test_resave!(parse_css_21,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style>#g1 { fill:red }</style>
    <style type='text/css'>#g1 { fill:blue }</style>
    <style type='blah'>#g1 { fill:red }</style>
    <g id='g1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='g1' fill='#0000ff'/>
</svg>
");

// marker property
test_resave!(parse_css_23,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style type='text/css'>
        rect { marker: url(#marker1); }
    </style>
    <marker id='marker1'/>
    <rect/>
</svg>
",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <marker id='marker1'/>
    <rect marker-end='url(#marker1)' marker-mid='url(#marker1)' marker-start='url(#marker1)'/>
</svg>
");

// no `type`
test_resave!(parse_css_24,
"<svg xmlns='http://www.w3.org/2000/svg'>
    <style>
    <![CDATA[
        .fil1 {fill:blue}
    ]]>
    </style>
    <g class='fil1'/>
</svg>",
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g fill='#0000ff'/>
</svg>
");
