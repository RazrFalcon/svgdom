// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate svgdom;

use svgdom::{
    AttributeId as AId,
    AttributeValue,
    Document,
    ElementId as EId,
    WriteOptions,
    ToStringWithOptions,
    ChainedErrorExt,
};

#[test]
fn linked_attributes_1() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    doc.root().append(&n1);
    doc.root().append(&n2);

    n2.set_id("2");

    n1.set_attribute((AId::XlinkHref, n2.clone()));

    assert_eq!(n1.is_used(), false);
    assert_eq!(n2.is_used(), true);

    assert_eq!(n2.linked_nodes().next().unwrap(), n1);
}

#[test]
fn linked_attributes_2() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    n1.set_id("1");
    n2.set_id("2");

    doc.root().append(&n1);
    doc.root().append(&n2);

    n1.set_attribute((AId::XlinkHref, n2.clone()));

    // recursion error
    assert_eq!(n2.set_attribute_checked((AId::XlinkHref, n1.clone())).unwrap_err().full_chain(),
               "Error: element crosslink");
}

#[test]
fn linked_attributes_3() {
    let mut doc = Document::new();

    {
        let mut n1 = doc.create_element(EId::Svg);
        let mut n2 = doc.create_element(EId::Svg);

        doc.root().append(&n1);
        doc.root().append(&n2);

        n1.set_id("1");
        n2.set_id("2");

        n1.set_attribute((AId::XlinkHref, n2.clone()));

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n1
        let mut n = doc.descendants().next().unwrap();
        n.remove();
    }

    {
        // n2 should became unused
        let n = doc.descendants().next().unwrap();
        assert_eq!(n.is_used(), false);
    }
}

#[test]
fn linked_attributes_4() {
    let mut doc = Document::new();

    {
        let mut n1 = doc.create_element(EId::Svg);
        let mut n2 = doc.create_element(EId::Svg);

        doc.root().append(&n1);
        doc.root().append(&n2);

        n1.set_id("1");
        n2.set_id("2");

        n1.set_attribute((AId::XlinkHref, n2.clone()));

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n2
        let mut n = doc.descendants().nth(1).unwrap();
        n.remove();
    }

    {
        // xlink:href attribute from n1 should be removed
        let n = doc.descendants().next().unwrap();
        assert_eq!(n.has_attribute(AId::XlinkHref), false);
    }
}

#[test]
fn linked_attributes_5() {
    let mut doc = Document::new();
    let mut n1 = doc.create_element(EId::Svg);
    let mut n2 = doc.create_element(EId::Svg);

    doc.root().append(&n1);
    doc.root().append(&n2);

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
"<svg>
    <rect/>
</svg>").unwrap();

    assert_eq!(doc.drain(|n| n.is_tag_name(EId::Rect)), 1);

    assert_eq_text!(doc.to_string(), "<svg/>\n");
}

#[test]
fn drain_2() {
    let mut doc = Document::from_str(
"<svg>
    <rect/>
    <g>
        <path/>
    </g>
    <rect/>
</svg>").unwrap();

    assert_eq!(doc.drain(|n| n.is_tag_name(EId::Path)), 1);

    assert_eq_text!(doc.to_string(),
"<svg>
    <rect/>
    <g/>
    <rect/>
</svg>
");
}

#[test]
fn drain_3() {
    let mut doc = Document::from_str(
"<svg>
    <rect/>
    <g>
        <path/>
    </g>
    <rect/>
</svg>").unwrap();

    assert_eq!(doc.drain(|n| n.is_tag_name(EId::G)), 1);

    assert_eq_text!(doc.to_string(),
"<svg>
    <rect/>
    <rect/>
</svg>
");
}

#[test]
fn drain_4() {
    let mut doc = Document::from_str(
"<svg>
    <rect/>
    <g>
        <rect/>
    </g>
    <rect/>
</svg>").unwrap();

    assert_eq!(doc.drain(|n| n.is_tag_name(EId::Rect)), 3);

    assert_eq_text!(doc.to_string(),
"<svg>
    <g/>
</svg>
");
}

#[test]
fn parents_1() {
    let doc = Document::from_str(
"<svg>
    <rect/>
    <g>
        <path/>
    </g>
    <rect/>
</svg>").unwrap();

    let node = doc.descendants().filter(|n| n.is_tag_name(EId::Path)).nth(0).unwrap();

    let mut iter = node.parents();
    assert_eq!(iter.next().unwrap().is_tag_name(EId::G), true);
    assert_eq!(iter.next().unwrap().is_tag_name(EId::Svg), true);
    assert_eq!(iter.next(), None);
}

#[test]
fn parents_2() {
    let doc = Document::from_str(
"<svg>
    <!--comment-->
    <g>
        <!--comment-->
        <g>
            <text>
                text1
                <tspan>text2</tspan>
            </text>
        </g>
    </g>
</svg>").unwrap();

    let node = doc.descendants().filter(|n| n.is_tag_name(EId::Tspan)).nth(0).unwrap();

    let mut iter = node.parents();
    assert_eq!(iter.next().unwrap().is_tag_name(EId::Text), true);
    assert_eq!(iter.next().unwrap().is_tag_name(EId::G), true);
    assert_eq!(iter.next().unwrap().is_tag_name(EId::G), true);
    assert_eq!(iter.next().unwrap().is_tag_name(EId::Svg), true);
    assert_eq!(iter.next(), None);
}

#[test]
fn deep_copy_1() {
    let doc = Document::from_str(
"<svg>
    <g id='g1'>
        <rect id='rect1'/>
    </g>
</svg>").unwrap();

    let mut svg = doc.svg_element().unwrap();
    let g = doc.descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // simple copy
    svg.append(&g.make_deep_copy());

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
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
    let doc = Document::from_str(
"<svg>
    <g id='g1'>
        <rect id='rect1'/>
    </g>
</svg>").unwrap();

    let mut g = doc.descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // copy itself
    let g1 = g.make_deep_copy();
    g.append(&g1);
    let g2 = g.make_deep_copy();
    g.append(&g2);

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
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
    let doc = Document::from_str(
"<svg>
    <linearGradient id='lg1'/>
    <g id='g1' stroke-width='5'>
        <rect id='rect1' fill='url(#lg1)'/>
    </g>
</svg>").unwrap();

    let mut svg = doc.svg_element().unwrap();
    let g = doc.descendants().find(|n| n.is_tag_name(EId::G)).unwrap();

    // test attributes copying
    svg.append(&g.make_deep_copy());

    let mut opt = WriteOptions::default();
    opt.use_single_quote = true;
    assert_eq_text!(doc.to_string_with_opt(&opt),
"<svg>
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

    rect.set_attribute((AId::XlinkHref, rect2));
    assert_eq!(rect.attributes().get(AId::XlinkHref).unwrap().to_string(), "xlink:href=\"#rect2\"");
}

#[test]
#[should_panic]
fn set_attr_2() {
    let mut doc = Document::new();
    let mut rect = doc.create_element(EId::Rect);
    let mut rect2 = doc.create_element(EId::Rect);
    rect2.set_id("rect2");

    rect.set_attribute((AId::XlinkHref, rect2));
    let attr = rect.attributes().get(AId::XlinkHref).cloned().unwrap();

    // must panic
    rect.attributes_mut().insert(attr);
}

#[test]
#[should_panic]
fn remove_attr_1() {
    let mut doc = Document::new();
    let mut rect = doc.create_element(EId::Rect);
    let mut rect2 = doc.create_element(EId::Rect);
    rect2.set_id("rect2");

    rect.set_attribute((AId::XlinkHref, rect2));

    // must panic
    rect.attributes_mut().remove(AId::XlinkHref);
}

#[test]
#[should_panic]
fn remove_attr_2() {
    let mut doc = Document::new();
    let mut rect = doc.create_element(EId::Rect);
    let mut rect2 = doc.create_element(EId::Rect);
    rect2.set_id("rect2");

    rect.set_attribute((AId::XlinkHref, rect2));

    // must panic
    rect.attributes_mut().retain(|a| !a.has_id(AId::XlinkHref));
}
