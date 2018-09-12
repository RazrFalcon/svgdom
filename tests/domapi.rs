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
    WriteOptions,
    WriteBuffer,
};

#[test]
fn linked_attributes_1() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    doc.root().append(n1.clone());
    doc.root().append(n2.clone());

    n2.set_id("2");

    n1.set_attribute((AId::Href, n2.clone()));

    assert_eq!(n1.is_used(), false);
    assert_eq!(n2.is_used(), true);

    assert_eq!(*n2.linked_nodes().iter().next().unwrap(), n1);
}

#[test]
fn linked_attributes_2() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    n1.set_id("1");
    n2.set_id("2");

    doc.root().append(n1.clone());
    doc.root().append(n2.clone());

    n1.set_attribute((AId::Href, n2.clone()));

    // recursion error
    assert_eq!(n2.set_attribute_checked((AId::Href, n1.clone())).unwrap_err().to_string(),
               "element crosslink");
}

#[test]
fn linked_attributes_3() {
    let mut doc = Document::new();

    {
        let mut n1 = doc.create_element(EId::Svg);
        let mut n2 = doc.create_element(EId::Svg);

        doc.root().append(n1.clone());
        doc.root().append(n2.clone());

        n1.set_id("1");
        n2.set_id("2");

        n1.set_attribute((AId::Href, n2.clone()));

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n1
        let n = doc.root().descendants().skip(1).next().unwrap();
        assert_eq!(*n.id(), "1");
        doc.remove_node(n);
    }

    {
        // n2 should became unused
        let n = doc.root().descendants().skip(1).next().unwrap();
        assert_eq!(*n.id(), "2");
        assert_eq!(n.is_used(), false);
    }
}

#[test]
fn linked_attributes_4() {
    let mut doc = Document::new();

    {
        let mut n1 = doc.create_element(EId::Svg);
        let mut n2 = doc.create_element(EId::Svg);

        doc.root().append(n1.clone());
        doc.root().append(n2.clone());

        n1.set_id("1");
        n2.set_id("2");

        n1.set_attribute((AId::Href, n2.clone()));

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n2
        let n = doc.root().descendants().nth(1).unwrap();
        doc.remove_node(n);
    }

    {
        // xlink:href attribute from n1 should be removed
        let n = doc.root().descendants().next().unwrap();
        assert_eq!(n.has_attribute(AId::Href), false);
    }
}

#[test]
fn linked_attributes_5() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    doc.root().append(n1.clone());
    doc.root().append(n2.clone());

    n1.set_id("1");
    n2.set_id("2");

    // no matter how many times we insert/clone/link same node,
    // amount of linked nodes in n1 must be 1
    n2.set_attribute((AId::Fill, n1.clone()));
    n2.set_attribute((AId::Fill, n1.clone()));
    n2.set_attribute((AId::Fill, n1.clone()));
    n2.set_attribute((AId::Fill, n1.clone()));

    assert_eq!(n1.is_used(), true);
    assert_eq!(n2.is_used(), false);

    assert_eq!(n1.uses_count(), 1);
}

#[test]
fn linked_attributes_6() {
    // Linked nodes not added to the tree should not cause a memory leak.

    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    n1.set_id("1");
    n2.set_id("2");

    n2.set_attribute((AId::Fill, n1.clone()));
}

#[test]
fn attributes_must_be_uniq() {
    let mut doc = Document::new();
    let mut n = doc.create_element(EId::Svg);

    n.set_attribute((AId::Fill, "red"));
    n.set_attribute((AId::Fill, "green"));

    assert_eq!(n.attributes().get_value(AId::Fill).unwrap(), &AttributeValue::from("green"));
    assert_eq!(n.attributes().len(), 1);
}

#[test]
fn attributes_compare_1() {
    let mut doc = Document::new();
    let mut n = doc.create_element(EId::Svg);

    n.set_attribute((AId::StrokeWidth, 1.0));

    assert_eq!(n.attributes().get_value(AId::StrokeWidth).unwrap(), &AttributeValue::from(1.0));
}

#[test]
fn attributes_exist_1() {
    let mut doc = Document::new();
    let mut n = doc.create_element(EId::Svg);

    n.set_attribute((AId::StrokeWidth, 1.0));

    assert_eq!(n.has_attribute(AId::StrokeWidth), true);
}

#[test]
fn attributes_exist_2() {
    let mut doc = Document::new();
    let mut n = doc.create_element(EId::Svg);

    n.set_attribute((AId::StrokeWidth, 1.0));

    assert_eq!(n.attributes().iter().find(|ref attr| attr.has_id(AId::StrokeWidth)).is_some(), true);
}

#[test]
fn remove_attribute_1() {
    let mut doc = Document::new();
    let mut n = doc.create_element(EId::Svg);

    n.set_attribute((AId::StrokeWidth, 1.0));
    assert_eq!(n.has_attribute(AId::StrokeWidth), true);

    n.remove_attribute(AId::StrokeWidth);
    assert_eq!(n.has_attribute(AId::StrokeWidth), false);
}

#[test]
fn drain_1() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
</svg>").unwrap();

    let root = doc.root().clone();
    assert_eq!(doc.drain(root, |n| n.is_tag_name(EId::Rect)), 1);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
               "<svg xmlns='http://www.w3.org/2000/svg'/>\n");
}

#[test]
fn drain_2() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
    <g>
        <path/>
    </g>
    <rect/>
</svg>").unwrap();

    let root = doc.root().clone();
    assert_eq!(doc.drain(root, |n| n.is_tag_name(EId::Path)), 1);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
    <g/>
    <rect/>
</svg>
");
}

#[test]
fn drain_3() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
    <g>
        <path/>
    </g>
    <rect/>
</svg>").unwrap();

    let root = doc.root().clone();
    assert_eq!(doc.drain(root, |n| n.is_tag_name(EId::G)), 1);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
    <rect/>
</svg>
");
}

#[test]
fn drain_4() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <rect/>
    <g>
        <rect/>
    </g>
    <rect/>
</svg>").unwrap();

    let root = doc.root().clone();
    assert_eq!(doc.drain(root, |n| n.is_tag_name(EId::Rect)), 3);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g/>
</svg>
");
}

#[test]
fn deep_copy_1() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='g1'>
        <rect id='rect1'/>
    </g>
</svg>").unwrap();

    let mut svg = doc.svg_element().unwrap();
    let g = doc.root().descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // simple copy
    svg.append(doc.copy_node_deep(g));

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='g1'>
        <rect id='rect1'/>
    </g>
    <g>
        <rect/>
    </g>
</svg>
");
}

#[test]
fn deep_copy_2() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='g1'>
        <rect id='rect1'/>
    </g>
</svg>").unwrap();

    let mut g = doc.root().descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // copy itself
    let g1 = doc.copy_node_deep(g.clone());
    g.append(g1);
    let g2 = doc.copy_node_deep(g.clone());
    g.append(g2);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <g id='g1'>
        <rect id='rect1'/>
        <g>
            <rect/>
        </g>
        <g>
            <rect/>
            <g>
                <rect/>
            </g>
        </g>
    </g>
</svg>
");
}

#[test]
fn deep_copy_3() {
    let mut doc = Document::from_str(
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient id='lg1'/>
    <g id='g1' stroke-width='5'>
        <rect id='rect1' fill='url(#lg1)'/>
    </g>
</svg>").unwrap();

    let mut svg = doc.svg_element().unwrap();
    let g = doc.root().descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // test attributes copying
    svg.append(doc.copy_node_deep(g));

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq!(doc.with_write_opt(&opt).to_string(),
"<svg xmlns='http://www.w3.org/2000/svg'>
    <linearGradient id='lg1'/>
    <g id='g1' stroke-width='5'>
        <rect id='rect1' fill='url(#lg1)'/>
    </g>
    <g stroke-width='5'>
        <rect fill='url(#lg1)'/>
    </g>
</svg>
");
}

#[test]
fn set_attr_1() {
    use svgdom::Attribute;

    let mut doc = Document::new();
    let mut rect = doc.create_element(EId::Rect);
    let mut rect2 = doc.create_element(EId::Rect);
    rect2.set_id("rect2");

    rect.set_attribute((AId::X, 1.0));
    assert_eq!(rect.attributes().get(AId::X).unwrap().to_string(), "x=\"1\"");

    rect.set_attribute(("attr", 1.0));
    assert_eq!(rect.attributes().get("attr").unwrap().to_string(), "attr=\"1\"");

    let attr = Attribute::new(AId::Y, 1.0);
    rect.set_attribute(attr);
    assert_eq!(rect.attributes().get(AId::Y).unwrap().to_string(), "y=\"1\"");

    rect.set_attribute((AId::Href, rect2));
    assert_eq!(rect.attributes().get(AId::Href).unwrap().to_string(), "xlink:href=\"#rect2\"");
}
