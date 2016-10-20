// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate svgdom;

use svgdom::{Document, AttributeValue, Error};
use svgdom::AttributeId as AId;
use svgdom::ElementId as EId;

#[test]
fn linked_attributes_1() {
    let doc = Document::new();
    let n1 = doc.create_element(EId::Svg);
    let n2 = doc.create_element(EId::Svg);

    doc.root().append(&n1);
    doc.root().append(&n2);

    n2.set_id("2");

    n1.set_link_attribute(AId::XlinkHref, n2.clone()).unwrap();

    assert_eq!(n1.is_used(), false);
    assert_eq!(n2.is_used(), true);

    assert_eq!(n2.linked_nodes().next().unwrap(), n1);
}

#[test]
fn linked_attributes_2() {
    let doc = Document::new();
    let n1 = doc.create_element(EId::Svg);
    let n2 = doc.create_element(EId::Svg);

    n1.set_id("1");
    n2.set_id("2");

    doc.root().append(&n1);
    doc.root().append(&n2);

    n1.set_link_attribute(AId::XlinkHref, n2.clone()).unwrap();

    // recursion error
    assert_eq!(n2.set_link_attribute(AId::XlinkHref, n1.clone()).unwrap_err(),
               Error::ElementCrosslink);
}

#[test]
fn linked_attributes_3() {
    let doc = Document::new();

    {
        let n1 = doc.create_element(EId::Svg);
        let n2 = doc.create_element(EId::Svg);

        doc.root().append(&n1);
        doc.root().append(&n2);

        n1.set_id("1");
        n2.set_id("2");

        n1.set_link_attribute(AId::XlinkHref, n2.clone()).unwrap();

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n1
        let n = doc.descendants().next().unwrap();
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
    let doc = Document::new();

    {
        let n1 = doc.create_element(EId::Svg);
        let n2 = doc.create_element(EId::Svg);

        doc.root().append(&n1);
        doc.root().append(&n2);

        n1.set_id("1");
        n2.set_id("2");

        n1.set_link_attribute(AId::XlinkHref, n2.clone()).unwrap();

        assert_eq!(n1.is_used(), false);
        assert_eq!(n2.is_used(), true);
    }

    {
        // remove n2
        let n = doc.descendants().nth(1).unwrap();
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
    let doc = Document::new();
    let n1 = doc.create_element(EId::Svg);
    let n2 = doc.create_element(EId::Svg);

    doc.root().append(&n1);
    doc.root().append(&n2);

    n1.set_id("1");
    n2.set_id("2");

    // no matter how many times we insert/clone/link same node,
    // amount of linked nodes in n1 must be 1
    n2.set_link_attribute(AId::Fill, n1.clone()).unwrap();
    n2.set_link_attribute(AId::Fill, n1.clone()).unwrap();
    n2.set_link_attribute(AId::Fill, n1.clone()).unwrap();
    n2.set_link_attribute(AId::Fill, n1.clone()).unwrap();

    assert_eq!(n1.is_used(), true);
    assert_eq!(n2.is_used(), false);

    assert_eq!(n1.uses_count(), 1);
}

#[test]
fn attributes_must_be_uniq() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::Fill, "red");
    n.set_attribute(AId::Fill, "green");

    assert_eq!(n.attribute_value(AId::Fill).unwrap(), AttributeValue::from("green"));
    assert_eq!(n.attributes().len(), 1);
}

#[test]
fn attributes_compare_1() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::StrokeWidth, 1.0);

    assert_eq!(n.attribute_value(AId::StrokeWidth).unwrap(), AttributeValue::from(1.0));
}

#[test]
fn attributes_compare_2() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::StrokeWidth, 1.0);

    assert_eq!(n.has_attribute_with_value(AId::StrokeWidth, 1.0), true);
}

#[test]
fn attributes_exist_1() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::StrokeWidth, 1.0);

    assert_eq!(n.has_attribute(AId::StrokeWidth), true);
}

#[test]
fn attributes_exist_2() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::StrokeWidth, 1.0);

    assert_eq!(n.attributes().iter().find(|ref attr| attr.has_id(AId::StrokeWidth)).is_some(), true);
}

#[test]
fn remove_attribute_1() {
    let doc = Document::new();
    let n = doc.create_element(EId::Svg);

    n.set_attribute(AId::StrokeWidth, 1.0);
    assert_eq!(n.has_attribute(AId::StrokeWidth), true);

    n.remove_attribute(AId::StrokeWidth);
    assert_eq!(n.has_attribute(AId::StrokeWidth), false);
}

#[test]
fn drain_1() {
    let doc = Document::from_data(
                b"<svg>
                    <rect/>
                </svg>").unwrap();

    assert_eq!(doc.root().drain(|n| n.is_tag_id(EId::Rect)), 1);

    assert_eq_text!(doc.to_string(), "<svg/>\n");
}

#[test]
fn drain_2() {
    let doc = Document::from_data(
                b"<svg>
                    <rect/>
                    <g>
                        <path/>
                    </g>
                    <rect/>
                </svg>").unwrap();

    assert_eq!(doc.root().drain(|n| n.is_tag_id(EId::Path)), 1);

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
    let doc = Document::from_data(
                b"<svg>
                    <rect/>
                    <g>
                        <path/>
                    </g>
                    <rect/>
                </svg>").unwrap();

    assert_eq!(doc.root().drain(|n| n.is_tag_id(EId::G)), 1);

    assert_eq_text!(doc.to_string(),
"<svg>
    <rect/>
    <rect/>
</svg>
");
}

#[test]
fn drain_4() {
    let doc = Document::from_data(
                b"<svg>
                    <rect/>
                    <g>
                        <rect/>
                    </g>
                    <rect/>
                </svg>").unwrap();

    assert_eq!(doc.root().drain(|n| n.is_tag_id(EId::Rect)), 3);

    assert_eq_text!(doc.to_string(),
"<svg>
    <g/>
</svg>
");
}
