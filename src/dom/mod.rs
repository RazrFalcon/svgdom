// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::iter::FilterMap;

pub use self::document::Document;
pub use self::element_type::ElementType;
pub use self::node::Node;
pub use self::tree::*;

use {Attributes, QName, QNameRef, ElementId};

/// Type alias for `QNameRef<ElementId>`.
pub type TagNameRef<'a> = QNameRef<'a, ElementId>;
/// Type alias for `QName<ElementId>`.
pub type TagName = QName<ElementId>;


mod tree;
mod document;
mod element_type;
mod node;


/// An iterator over SVG elements.
pub trait FilterSvg: Iterator {
    /// Filters SVG elements.
    fn svg(self) -> FilterMap<Self, fn(Node) -> Option<(ElementId, Node)>>
        where Self: Iterator<Item = Node> + Sized,
    {
        fn is_svg(node: Node) -> Option<(ElementId, Node)> {
            if let QName::Id(_, id) = *node.tag_name() {
                return Some((id, node.clone()));
            }

            None
        }

        self.filter_map(is_svg)
    }
}

impl<I: Iterator> FilterSvg for I {}

/// List of supported node types.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum NodeType {
    /// Root node of the `Document`.
    ///
    /// Constructed with `Document`. Unavailable to the user.
    Root,
    /// Element node.
    ///
    /// Only an element can have attributes, ID and tag name.
    Element,
    /// Declaration node.
    Declaration,
    /// Comment node.
    Comment,
    /// CDATA node.
    Cdata,
    /// Text node.
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

impl fmt::Debug for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            NodeType::Root => write!(f, "RootNode"),
            NodeType::Element => write!(f, "ElementNode({:?} id={:?})", self.tag_name, self.id),
            NodeType::Declaration => write!(f, "DeclarationNode({:?})", self.text),
            NodeType::Comment => write!(f, "CommentNode({:?})", self.text),
            NodeType::Cdata => write!(f, "CdataNode({:?})", self.text),
            NodeType::Text => write!(f, "TextNode({:?})", self.text),
        }
    }
}
