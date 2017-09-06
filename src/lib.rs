// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
This library is designed to represent SVG data as a tree structure.

Here is simple overview of such structure:

- [`Document`]
    - root [`Node`]
        - user defined [`Node`]
            - [`TagName`]
            - [`Attributes`]
            - unique id
        - user defined [`Node`]
        - ...

The [`Document`] itself is just a container of [`Node`]s.
You can create new [`Node`]s only through the [`Document`]. Parsing and generating of the SVG data also
done through it.

The [`Node`] represents any kind of an XML node.
It can be an element, a comment, a text, etc. There are no different structs for each type.

The [`TagName`] represents a tag name of the element node. It's an enum of
[`ElementId`] and `String` types. The [`ElementId`] contains all possible
SVG element names and `String` used for non-SVG elements. Such separation used for
performance reasons.

The [`Attributes`] container wraps a `Vec` of [`Attribute`]'s.

At last, the `id` attribute is stored as a separate value and not as part of the [`Attributes`].

&nbsp;

See modules and structs documentation for details.

&nbsp;

DOM structure itself based on: https://github.com/SimonSapin/rust-forest/tree/master/rctree

[`Attribute`]: struct.Attribute.html
[`Attributes`]: struct.Attributes.html
[`Document`]: struct.Document.html
[`ElementId`]: enum.ElementId.html
[`Node`]: struct.Node.html
[`TagName`]: type.TagName.html

*/

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(collapsible_if))]
#![cfg_attr(feature="clippy", allow(module_inception))]
#![cfg_attr(feature="clippy", allow(new_without_default))]
#![cfg_attr(feature="clippy", allow(new_without_default_derive))]

#[macro_use]
extern crate svgparser;
extern crate simplecss;
extern crate float_cmp;

pub use attribute::*;
pub use dom::*;
pub use error::Error;
pub use name::*;
pub use traits::*;
pub use writer::{
    WriteOptions,
    WriteOptionsPaths,
    Indent
};

#[cfg(feature = "parsing")]
pub use parser::ParseOptions;

pub use svgparser::AttributeId;
pub use svgparser::ElementId;
pub use svgparser::ErrorPos;
pub use svgparser::ValueId;

#[macro_use]
mod traits;

// TODO: #[cfg(test)]
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
mod dom;
mod error;
mod name;
mod writer;

#[cfg(feature = "parsing")]
mod parser;

pub mod types;
pub mod postproc;
