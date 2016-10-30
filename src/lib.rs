// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
This library is designed to represent SVG data as a tree structure.

Here is simple overview of such structure:

- [`Document`](struct.Document.html)
    - root [`Node`](struct.Node.html)
        - user defined [`Node`](struct.Node.html)
            - [`TagName`](struct.TagName.html)
            - [`Attributes`](struct.Attributes.html)
            - unique id
        - user defined [`Node`](struct.Node.html)
        - ...

The [`Document`](struct.Document.html) itself is just a container of the `Node`s.
You can create new `Node`s only by the `Document`. Parsing and generating of the SVG data also
done through it.

The [`Node`](struct.Node.html) represents any kind of an XML node.
It can be an element, a comment, a text, etc. There are no different structs for each type.

The [`TagName`](struct.TagName.html) represents a tag name of the element node. It's an enum of
[`ElementId`](enum.ElementId.html) and `String` types. The `ElementId` contains all possible
SVG element names and `String` used for non-SVG elements. Such separation used for
performance reasons.

The [`Attributes`](struct.Attributes.html) container wraps a `Vec` of
[`Attribute`](struct.Attribute.html)'s.

At last, the `id` attribute is stored as a separate value and not as part of the `Attributes`.

&nbsp;

See modules and structs documentation for details.

&nbsp;

DOM structure itself based on: https://github.com/SimonSapin/rust-forest/tree/master/rctree
*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[macro_use]
extern crate svgparser;
extern crate multimap;

pub use attribute::*;
pub use attribute_value::AttributeValue;
pub use attributes::Attributes;
pub use dom::*;
pub use error::Error;
pub use name::*;
pub use parse_options::*;
pub use traits::*;
pub use write_options::*;

pub use svgparser::AttributeId;
pub use svgparser::ElementId;
pub use svgparser::ErrorPos;
pub use svgparser::ValueId;

#[macro_use]
mod traits;

#[macro_export]
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

mod attribute;
mod attribute_value;
mod attributes;
mod dom;
mod error;
mod name;
mod parse_options;
mod parser;
mod write_options;

pub mod types;
pub mod writer;
