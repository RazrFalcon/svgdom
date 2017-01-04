// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[cfg(feature = "parsing")]
use parser::parse_svg;
#[cfg(feature = "parsing")]
use {ParseOptions, Error};

use writer;
use {
    Attributes,
    ElementId,
    NameRef,
    TagName,
    TagNameRef,
    WriteBuffer,
    WriteOptions,
    WriteToString,
};

use super::iterators::{Children, Descendants};
use super::node::Node;
use super::node_data::{Link, NodeData};
use super::node_type::NodeType;

/// Container of [`Node`](struct.Node.html)s.
pub struct Document {
    /// Root node.
    pub root: Node,
}

impl Document {
    /// Constructs a new `Document`.
    pub fn new() -> Document {
        Document {
            root: Document::new_node(None, NodeType::Root, None, None)
        }
    }

    #[cfg(feature = "parsing")]
    /// Constructs a new `Document` from the `data` using a default `ParseOptions`.
    pub fn from_data(data: &[u8]) -> Result<Document, Error> {
        Document::from_data_with_opt(data, &ParseOptions::default())
    }

    #[cfg(feature = "parsing")]
    /// Constructs a new `Document` from the `data` using a supplied `ParseOptions`.
    pub fn from_data_with_opt(data: &[u8], opt: &ParseOptions) -> Result<Document, Error> {
        parse_svg(data, opt)
    }

    /// Constructs a new `Node` with `Element` type.
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    ///
    /// # Panics
    ///
    /// Panics if a string tag name is empty.
    pub fn create_element<'a, T>(&self, tag_name: T) -> Node
        where TagNameRef<'a>: From<T>
    {
        let tn = TagNameRef::from(tag_name);
        if let NameRef::Name(ref name) = tn {
            if name.is_empty() {
                panic!("supplied tag name is empty");
            }
        }

        Document::new_node(Some(self.root.0.clone()), NodeType::Element, Some(tn), None)
    }

    /// Constructs a new `Node` using the supplied `NodeType`.
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    ///
    /// This method should be used for any non-element nodes.
    pub fn create_node(&self, node_type: NodeType, text: &str) -> Node {
        debug_assert!(node_type != NodeType::Element && node_type != NodeType::Root);
        Document::new_node(Some(self.root.0.clone()), node_type, None, Some(text.to_owned()))
    }

    /// Returns the root `Node`.
    pub fn root(&self) -> Node {
        self.root.clone()
    }

    /// Returns the first child of the root `Node`.
    ///
    /// # Panics
    ///
    /// Panics if the root node is currently mutability borrowed.
    pub fn first_child(&self) -> Option<Node> {
        self.root().first_child()
    }

    /// Returns the first child with `svg` tag name of the root `Node`.
    ///
    /// In most of the cases result of this method and `first_element_child()` will be the same,
    /// but an additional check may be helpful.
    ///
    /// # Panics
    ///
    /// Panics if the root node is currently mutability borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::{Document, ElementId};
    ///
    /// let doc = Document::from_data(b"<!--comment--><svg/>").unwrap();
    ///
    /// assert_eq!(doc.svg_element().unwrap().is_tag_name(ElementId::Svg), true);
    /// ```
    pub fn svg_element(&self) -> Option<Node> {
        for n in self.root.children().svg() {
            if n.is_tag_name(ElementId::Svg) {
                return Some(n.clone());
            }
        }

        None
    }

    /// Appends a new child to root node, after existing children, and returns it.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new child, or one of their adjoining nodes is currently borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::{Document, ElementId};
    ///
    /// let doc = Document::new();
    /// doc.append(&doc.create_element(ElementId::Svg));
    ///
    /// assert_eq!(doc.to_string(), "<svg/>\n");
    /// ```
    pub fn append(&self, new_child: &Node) -> Node {
        self.root.append(new_child);
        new_child.clone()
    }

    /// Returns an iterator over descendants.
    pub fn descendants(&self) -> Descendants {
        self.root.descendants()
    }

    /// Returns an iterator to this node's children elements.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn children(&self) -> Children {
        self.root.children()
    }

    /// Removes only the children nodes specified by the predicate.
    ///
    /// The root node ignored.
    pub fn drain<P>(&self, f: P) -> usize
        where P: Fn(&Node) -> bool
    {
        self.root().drain(f)
    }

    fn new_node(doc: Option<Link>, node_type: NodeType, tag_name: Option<TagNameRef>,
                text: Option<String>)
                -> Node {
        Node(Rc::new(RefCell::new(NodeData {
            doc: doc,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            node_type: node_type,
            tag_name: tag_name.map(|a| TagName::from(a)),
            id: String::new(),
            attributes: Attributes::new(),
            linked_nodes: Vec::new(),
            text: text,
        })))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteBuffer for Document {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        writer::write_dom(self, opt, buf);
    }
}

impl_display!(Document);
