// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use {Document, Node, AttributeId, Attribute, AttributeValue, ValueId};

use error::Error;

// TODO: split
/// Resolve `inherit` and `currentColor` attributes.
///
/// The function will fallback to a default value when possible.
///
/// # Errors
///
/// Will return `Error::UnresolvedAttribute` if an attribute
/// can't be resolved and didn't have a default value.
pub fn resolve_inherit(doc: &Document) -> Result<(), Error> {
    let mut vec_inherit = Vec::new();
    let mut vec_curr_color = Vec::new();

    for node in doc.descendants().svg() {
        vec_inherit.clear();
        vec_curr_color.clear();

        {
            let attrs = node.attributes();
            for (aid, attr) in attrs.iter_svg() {
                if let AttributeValue::PredefValue(ref v) = attr.value {
                    match *v {
                        ValueId::Inherit => {
                            vec_inherit.push(aid);
                        }
                        ValueId::CurrentColor => {
                            vec_curr_color.push(aid);
                        }
                        _ => {},
                    }
                }
            }
        }

        for id in &vec_inherit {
            try!(resolve_impl(&node, *id, *id));
        }

        for id in &vec_curr_color {
            if let Some(av) = node.attribute_value(AttributeId::Color) {
                node.set_attribute(*id, av);
            } else {
                try!(resolve_impl(&node, *id, AttributeId::Color));
            }
        }
    }

    Ok(())
}

fn resolve_impl(node: &Node, curr_attr: AttributeId, parent_attr: AttributeId) -> Result<(), Error> {
    if let Some(n) = node.parents().find(|n| n.has_attribute(parent_attr)) {
        let av = n.attribute_value(parent_attr).unwrap();
        node.set_attribute(curr_attr, av);
    } else {
        match Attribute::default(curr_attr) {
            Some(a) => node.set_attribute(curr_attr, a.value),
            None => {
                return Err(Error::UnresolvedAttribute(curr_attr.name().to_string()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use {Document, WriteToString};

    macro_rules! test {
        ($name:ident, $in_text:expr, $out_text:expr) => (
            #[test]
            fn $name() {
                let doc = Document::from_data($in_text).unwrap();
                resolve_inherit(&doc).unwrap();
                assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
            }
        )
    }

    test!(inherit_1,
b"<svg fill='#ff0000'>
    <rect fill='inherit'/>
</svg>",
"<svg fill='#ff0000'>
    <rect fill='#ff0000'/>
</svg>
");

    test!(inherit_2,
b"<svg fill='#ff0000'>
    <g>
        <rect fill='inherit'/>
    </g>
</svg>",
"<svg fill='#ff0000'>
    <g>
        <rect fill='#ff0000'/>
    </g>
</svg>
");

    test!(inherit_3,
b"<svg fill='#ff0000' stroke='#00ff00'>
    <rect fill='inherit' stroke='inherit'/>
</svg>",
"<svg fill='#ff0000' stroke='#00ff00'>
    <rect fill='#ff0000' stroke='#00ff00'/>
</svg>
");

    test!(inherit_4,
b"<svg>
    <rect fill='inherit'/>
</svg>",
"<svg>
    <rect fill='#000000'/>
</svg>
");

    test!(current_color_1,
b"<svg color='#ff0000'>
    <rect fill='currentColor'/>
</svg>",
"<svg color='#ff0000'>
    <rect fill='#ff0000'/>
</svg>
");

    test!(current_color_2,
b"<svg>
    <rect color='#ff0000' fill='currentColor'/>
</svg>",
"<svg>
    <rect color='#ff0000' fill='#ff0000'/>
</svg>
");

    test!(current_color_3,
b"<svg color='#ff0000'>
    <rect fill='currentColor' stroke='currentColor'/>
</svg>",
"<svg color='#ff0000'>
    <rect fill='#ff0000' stroke='#ff0000'/>
</svg>
");

    test!(default_1,
b"<svg>
    <rect fill='currentColor'/>
    <rect fill='inherit'/>
</svg>",
"<svg>
    <rect fill='#000000'/>
    <rect fill='#000000'/>
</svg>
");
}
