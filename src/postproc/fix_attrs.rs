// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use {
    AttributeId,
    AttributeValue,
    ElementId,
    ElementType,
    Node,
};
use types::Length;

/// Fix `rect` element attributes.
///
/// - A negative `width` will be replaced with `0`
/// - A negative `height` will be replaced with `0`
/// - A negative `rx` will be removed
/// - A negative `ry` will be removed
///
/// Details: https://www.w3.org/TR/SVG/shapes.html#RectElement
pub fn fix_rect_attributes(node: &Node) {
    debug_assert!(node.is_tag_name(ElementId::Rect));

    fix_len(node, AttributeId::Width,  Length::zero());
    fix_len(node, AttributeId::Height, Length::zero());

    rm_negative_len(node, AttributeId::Rx);
    rm_negative_len(node, AttributeId::Ry);

    // TODO: check that 'rx <= widht/2' and 'ry <= height/2'
    // Remember: a radius attributes can have different units,
    // so we need can't compare them. Probably we can do this only
    // after converting all units to px, which is optional.
}

#[cfg(test)]
mod test_rect {
    use super::*;
    use {Document, ElementId, WriteToString};

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                for node in doc.descendants().svg().filter(|n| n.is_tag_name(ElementId::Rect)) {
                    fix_rect_attributes(&node);
                }
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    test!(fix_rect_1,
"<svg>
    <rect/>
    <rect width='-1' height='-1'/>
    <rect width='30'/>
    <rect height='40'/>
    <rect width='-30'/>
    <rect height='-40'/>
    <rect width='0'/>
    <rect height='0'/>
</svg>",
"<svg>
    <rect height='0' width='0'/>
    <rect height='0' width='0'/>
    <rect height='0' width='30'/>
    <rect height='40' width='0'/>
    <rect height='0' width='0'/>
    <rect height='0' width='0'/>
    <rect height='0' width='0'/>
    <rect height='0' width='0'/>
</svg>
");

    test!(fix_rect_2,
"<svg>
    <rect height='50' width='40'/>
    <rect height='50' rx='-5' width='40'/>
    <rect height='50' ry='-5' width='40'/>
    <rect height='50' rx='-5' ry='-5' width='40'/>
</svg>",
"<svg>
    <rect height='50' width='40'/>
    <rect height='50' width='40'/>
    <rect height='50' width='40'/>
    <rect height='50' width='40'/>
</svg>
");

}

/// Fix `polyline` and `polygon` element attributes.
///
/// - An empty `points` attribute will be removed
/// - A `points` attribute with an odd number of coordinates will be truncated by one coordinate
///
/// Details: https://www.w3.org/TR/SVG/shapes.html#PolylineElement
/// https://www.w3.org/TR/SVG/shapes.html#PolygonElement
pub fn fix_poly_attributes(node: &Node) {
    debug_assert!(node.is_tag_name(ElementId::Polyline) || node.is_tag_name(ElementId::Polygon));

    let mut attrs_data = node.attributes_mut();
    let mut is_empty = false;

    if let Some(points_value) = attrs_data.get_value_mut(AttributeId::Points) {
        if let AttributeValue::NumberList(ref mut p) = *points_value {
            if p.is_empty() {
                // remove if no points
                is_empty = true;
            } else if p.len() % 2 != 0 {
                // remove last point if points count is odd
                p.pop();

                // remove if no points
                if p.is_empty() {
                    is_empty = true;
                }
            }
        }
    }

    if is_empty {
        attrs_data.remove(AttributeId::Points);
    }
}

#[cfg(test)]
mod test_poly {
    use super::*;
    use {Document, ElementId, WriteToString};

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                for node in doc.descendants().svg()
                    .filter(|n| n.is_tag_name(ElementId::Polygon) || n.is_tag_name(ElementId::Polyline)) {
                    fix_poly_attributes(&node);
                }
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    test!(fix_polyline_1,
"<svg>
    <polyline points='5 6 7'/>
    <polyline points='5'/>
    <polyline points=''/>
    <polyline/>
</svg>",
"<svg>
    <polyline points='5 6'/>
    <polyline/>
    <polyline/>
    <polyline/>
</svg>
");

}

/// Fix `stop` element attributes.
///
/// - A negative `offset` will be replaced with `0`
/// - An `offset` value bigger than `1` will be replaced with `1`
/// - An `offset` value smaller that previous will be set to previous
///
/// This method accepts `Node` with `linearGradient` or `radialGradient` tag name.
///
/// You should run this function only after
/// [`resolve_stop_attributes()`](fn.resolve_stop_attributes.html).
///
/// Details: https://www.w3.org/TR/SVG/pservers.html#StopElementOffsetAttribute
pub fn fix_stop_attributes(node: &Node) {
    debug_assert!(node.is_gradient());

    let mut prev_offset = 0.0;

    for child in node.children() {
        // TODO: 'offset' must be resolved
        let mut offset = *child.attributes().get_value(AttributeId::Offset).unwrap()
                               .as_number().unwrap();

        if offset < 0.0 {
            offset = 0.0;
        } else if offset > 1.0 {
            offset = 1.0;
        }

        if offset < prev_offset {
            offset = prev_offset;
        }

        child.set_attribute(AttributeId::Offset, offset);

        prev_offset = offset;
    }
}

#[cfg(test)]
mod test_stop {
    use super::*;
    use {Document, WriteToString, ElementType};
    use postproc::resolve_stop_attributes;

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_str($in_text).unwrap();
                resolve_stop_attributes(&doc).unwrap();
                for node in doc.descendants().svg().filter(|n| n.is_gradient()) {
                    fix_stop_attributes(&node);
                }
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    test!(fix_stop_1,
"<svg>
    <linearGradient>
        <stop offset='-1'/>
        <stop offset='0.4'/>
        <stop offset='0.3'/>
        <stop offset='10'/>
        <stop offset='0.5'/>
    </linearGradient>
</svg>",
"<svg>
    <linearGradient>
        <stop offset='0'/>
        <stop offset='0.4'/>
        <stop offset='0.4'/>
        <stop offset='1'/>
        <stop offset='1'/>
    </linearGradient>
</svg>
");
}

fn fix_len(node: &Node, id: AttributeId, new_len: Length) {
    if node.has_attribute(id) {
        fix_negative_len(node, id, new_len);
    } else {
        node.set_attribute(id, new_len);
    }
}

fn fix_negative_len(node: &Node, id: AttributeId, new_len: Length) {
    let av = node.attributes().get_value(id).cloned();
    if let Some(av) = av {
        // unwrap is safe, because coordinates must have a Length type
        let l = av.as_length().unwrap();
        if l.num.is_sign_negative() {
            node.set_attribute(id, new_len);
        }
    }
}

fn rm_negative_len(node: &Node, id: AttributeId) {
    let av = node.attributes().get_value(id).cloned();
    if let Some(av) = av {
        // unwrap is safe, because coordinates must have a Length type
        let l = av.as_length().unwrap();
        if l.num.is_sign_negative() {
            node.remove_attribute(id);
        }
    }
}
