// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
This library is designed to represent SVG data as a tree structure.

Here is simple overview of a such structure:

- [`Document`](struct.Document.html)
    - root [`Node`](struct.Node.html)
        - user defined [`Node`](struct.Node.html)
            - [`TagName`](struct.TagName.html)
            - [`Attributes`](struct.Attribute.html)
            - non-SVG attributes
            - unique id
        - user defined [`Node`](struct.Node.html)
        - ...

The [`Document`](struct.Document.html) itself is just a container of the `Node`s.
You can create new a `Node`s only from the `Document`. Parsing and generating of the SVG data also
done through it.

The [`Node`](struct.Node.html) represents any kind of a XML node.
It can be an element, a comment, a text, etc. There are no different structs for each type.

The [`TagName`](struct.TagName.html) represents tag name of the element node. It's a tuple of
[`ElementId`](enum.ElementId.html) and `String` types. The `ElementId` contains all possible
SVG element names and `String` used for unknown elements. Such separation used for
a performance reasons.

There are two types of attributes, like with tag names: one for SVG attributes and one for unknown.
Unknown attributes stored in a simple `HashMap<String,String>` structure.
And SVG attributes stored behind a pretty complex struct.
See [`Attributes`](struct.Attribute.html) documentation for details.
Only SVG attributes supports [`AttributeValue`](enum.AttributeValue.html), which is stored
preprocessed data and not a raw strings like usual XML parser.

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

pub use attribute::{Attribute, AttributeValue};
pub use attributes::Attributes;
pub use dom::{Document, Node, NodeEdge, NodeType, TagName, Traverse};
pub use error::Error;
pub use parse_options::*;
pub use traits::*;
pub use write_options::*;

pub use svgparser::AttributeId;
pub use svgparser::ElementId;
pub use svgparser::ErrorPos;
pub use svgparser::ValueId;

#[macro_use]
mod traits;

#[cfg(test)]
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
mod attributes;
mod dom;
mod error;
mod parse_options;
mod parser;
mod write;
mod write_options;

pub mod types;
