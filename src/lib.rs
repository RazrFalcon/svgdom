// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

[`Attribute`]: struct.Attribute.html
[`Attributes`]: struct.Attributes.html
[`Document`]: struct.Document.html
[`ElementId`]: enum.ElementId.html
[`Node`]: type.Node.html
[`TagName`]: type.TagName.html

*/

#![doc(html_root_url = "https://docs.rs/svgdom/0.12.0")]

#![warn(missing_docs)]

#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate simplecss;
extern crate slab;
extern crate svgtypes;


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
mod parser;
mod writer;


pub use attribute::*;
pub use dom::*;
pub use error::Error;
pub use name::*;
pub use writer::*;
pub use parser::ParseOptions;

pub use svgtypes::{
    Align,
    AspectRatio,
    AttributeId,
    Color,
    ElementId,
    FromSpan,
    FuzzyEq,
    FuzzyZero,
    Length,
    LengthList,
    LengthUnit,
    ListSeparator,
    NumberList,
    Path,
    PathBuilder,
    PathCommand,
    PathSegment,
    Points,
    StrSpan,
    Transform,
    ViewBox,
    WriteBuffer as ValueWriteBuffer,
    WriteOptions as ValueWriteOptions,
};
