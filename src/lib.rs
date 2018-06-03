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

#![doc(html_root_url = "https://docs.rs/svgdom/0.13.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[macro_use] extern crate log;
extern crate simplecss;
extern crate slab;
extern crate svgtypes;


mod attribute;
mod document;
mod node;
mod tree;
mod element_type;
mod error;
mod name;
mod parser;
mod writer;
mod attribute_type;
mod attribute_value;
mod attributes;


pub use attribute::*;
pub use attribute_type::AttributeType;
pub use attribute_value::AttributeValue;
pub use attributes::*;
pub use document::Document;
pub use element_type::ElementType;
pub use error::*;
pub use name::*;
pub use node::*;
pub use parser::ParseOptions;
pub use tree::iterator::*;
pub use writer::*;

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
    PaintFallback,
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


/// Type alias for `QNameRef<ElementId>`.
pub type TagNameRef<'a> = QNameRef<'a, ElementId>;
/// Type alias for `QName<ElementId>`.
pub type TagName = QName<ElementId>;

/// Type alias for `QName<AttributeId>`.
pub type AttributeQName = QName<AttributeId>;
/// Type alias for `QNameRef<AttributeId>`.
pub type AttributeQNameRef<'a> = QNameRef<'a, AttributeId>;


/// List of supported node types.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeType {
    /// The root node of the `Document`.
    ///
    /// Constructed with `Document`. Unavailable to the user.
    Root,
    /// An element node.
    ///
    /// Only an element can have attributes, ID and tag name.
    Element,
    /// A declaration node.
    Declaration,
    /// A comment node.
    Comment,
    /// A CDATA node.
    Cdata,
    /// A text node.
    Text,
}


/// Node's data.
pub struct NodeData {
    storage_key: Option<usize>,
    node_type: NodeType,
    tag_name: TagName,
    id: String,
    attributes: Attributes,
    linked_nodes: Vec<Node>,
    text: String,
}
