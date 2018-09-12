// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!

*svgdom* is an [SVG Full 1.1](https://www.w3.org/TR/SVG/) processing library,
which allows you to parse, manipulate, generate and write an SVG content.

**Note:** the library itself is pretty stable, but API is constantly changing.

## Purpose

*svgdom* is designed to simplify generic SVG processing and manipulations.
Unfortunately, an SVG is very complex format (PDF spec is 826 pages long),
with lots of features and implementing all of them will lead to an enormous library.

That's why *svgdom* supports only a static subset of an SVG. No scripts, external resources
and complex CSS styling.
Parser will convert as much as possible data to a simple doc->elements->attributes structure.

For example, the `fill` parameter of an element can be set: as an element's attribute,
as part of a `style` attribute, inside a `style` element as CSS2, inside an `ENTITY`,
using a JS code and probably with lots of other methods.

Not to mention, that the `fill` attribute supports 4 different types of data.

With `svgdom` you can just use `node.has_attribute(AttributeId::Fill)` and don't worry where this
attribute was defined in the original file.

Same goes for transforms, paths and other SVG types.

The main downside of this approach is that you can't save an original formatting and some data.

See the [preprocessor](https://github.com/RazrFalcon/svgdom/blob/master/docs/preprocessor.md)
doc for details.

## Benefits

- The element link(IRI, FuncIRI) is not just a text, but an actual link to another node.
- At any time you can check which elements linked to the specific element.
  See `Node`'s doc for details.
- Support for many SVG specific data types like paths, transforms, IRI's, styles, etc.
  Thanks to [svgtypes](https://github.com/RazrFalcon/svgtypes).
- A complete support of text nodes: XML escaping, `xml:space`.
- Fine-grained control over the SVG output.

## Limitations

- Only SVG elements and attributes will be parsed.
- Attribute values, CDATA with CSS, DOCTYPE, text data and whitespaces will not be preserved.
- UTF-8 only.
- Only most popular attributes are parsed, other stored as strings.
- No compressed SVG (.svgz). You should decompress it by yourself.
- CSS support is minimal.
- SVG 1.1 Full only (no 2.0 Draft, Basic, Type subsets).

## Differences between svgdom and SVG spec

- Library follows SVG spec in the data parsing, writing, but not in the tree structure.
- Everything is a `Node`. There are no separated `ElementNode`, `TextNode`, etc.
  You still have all the data, but not in the specific *struct's*.
  You can check the node type via `Node::node_type()`.

*/

#![doc(html_root_url = "https://docs.rs/svgdom/0.14.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[macro_use] extern crate log;
extern crate simplecss;
extern crate slab;
extern crate svgtypes;
extern crate roxmltree;


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
    /// A comment node.
    Comment,
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
