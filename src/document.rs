// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

use slab::Slab;

use parser::parse_svg;
use {
    ParseOptions,
};

use writer;
use {
    AttributeQName,
    Attributes,
    AttributeValue,
    ElementId,
    FilterSvg,
    FilterSvgAttrs,
    Node,
    NodeData,
    NodeType,
    ParserError,
    QName,
    QNameRef,
    TagNameRef,
    WriteBuffer,
    WriteOptions,
};

/// Container of [`Node`]s.
///
/// Structure:
///
/// - [`Document`]
///     - root [`Node`]
///         - user defined [`Node`]
///             - [`TagName`]
///             - [`Attributes`]
///             - unique id
///         - user defined [`Node`]
///         - ...
///
/// The [`Document`] itself is just a container of [`Node`]s.
/// You can create new [`Node`]s only through the [`Document`].
/// Parsing and generating of the SVG data also done through it.
///
/// The [`Node`] represents any kind of an XML node.
/// It can be an element, a comment, a text, etc. There are no different structs for each type.
///
/// The [`TagName`] represents a tag name of the element node. It's an enum of
/// [`ElementId`] and `String` types. The [`ElementId`] contains all possible
/// SVG element names and `String` used for non-SVG elements. Such separation used for
/// performance reasons.
///
/// The [`Attributes`] container wraps a `Vec` of [`Attribute`]'s.
///
/// At last, the `id` attribute is stored as a separate value and not as part of the [`Attributes`].
///
/// [`Attribute`]: struct.Attribute.html
/// [`Attributes`]: struct.Attributes.html
/// [`Document`]: struct.Document.html
/// [`ElementId`]: enum.ElementId.html
/// [`Node`]: type.Node.html
/// [`TagName`]: type.TagName.html
pub struct Document {
    root: Node,
    storage: Slab<Node>,
}

impl Document {
    /// Constructs a new `Document`.
    pub fn new() -> Document {
        let mut storage = Slab::new();
        let mut root = Node::new(NodeData {
            storage_key: None,
            node_type: NodeType::Root,
            tag_name: QName::Name(String::new()),
            id: String::new(),
            attributes: Attributes::new(),
            linked_nodes: Vec::new(),
            text: String::new(),
        });

        let key = storage.insert(root.clone());
        root.borrow_mut().storage_key = Some(key);

        Document {
            root,
            storage,
        }
    }

    /// Constructs a new `Document` from the text using a default [`ParseOptions`].
    ///
    /// [`ParseOptions`]: struct.ParseOptions.html
    ///
    /// **Note:** only SVG elements and attributes will be parsed.
    pub fn from_str(text: &str) -> Result<Document, ParserError> {
        Document::from_str_with_opt(text, &ParseOptions::default())
    }

    /// Constructs a new `Document` from the text using a supplied [`ParseOptions`].
    ///
    /// [`ParseOptions`]: struct.ParseOptions.html
    ///
    /// **Note:** only SVG elements and attributes will be parsed.
    pub fn from_str_with_opt(text: &str, opt: &ParseOptions) -> Result<Document, ParserError> {
        parse_svg(text, opt)
    }

    /// Constructs a new [`Node`] with [`NodeType`]::Element type.
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    ///
    /// # Panics
    ///
    /// Panics if a string tag name is empty.
    ///
    /// [`Node`]: type.Node.html
    /// [`NodeType`]: enum.NodeType.html
    pub fn create_element<'a, T>(&mut self, tag_name: T) -> Node
        where TagNameRef<'a>: From<T>, T: Copy
    {
        let tn = QNameRef::from(tag_name);
        if let QNameRef::Name(name) = tn {
            if name.is_empty() {
                panic!("supplied tag name is empty");
            }
        }

        let mut node = Node::new(NodeData {
            storage_key: None,
            node_type: NodeType::Element,
            tag_name: QNameRef::from(tag_name).into(),
            id: String::new(),
            attributes: Attributes::new(),
            linked_nodes: Vec::new(),
            text: String::new(),
        });

        let key = self.storage.insert(node.clone());
        node.borrow_mut().storage_key = Some(key);

        node
    }

    // TODO: we can't have continuous text nodes.
    // TODO: doc should have only one declaration
    /// Constructs a new [`Node`] using the supplied [`NodeType`].
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    ///
    /// This method should be used for any non-element nodes.
    ///
    /// [`Node`]: type.Node.html
    /// [`NodeType`]: enum.NodeType.html
    pub fn create_node<S: Into<String>>(&mut self, node_type: NodeType, text: S) -> Node {
        assert!(node_type != NodeType::Element && node_type != NodeType::Root);

        let mut node = Node::new(NodeData {
            storage_key: None,
            node_type,
            tag_name: QName::Name(String::new()),
            id: String::new(),
            attributes: Attributes::new(),
            linked_nodes: Vec::new(),
            text: text.into(),
        });

        let key = self.storage.insert(node.clone());
        node.borrow_mut().storage_key = Some(key);

        node
    }

    /// Returns the root [`Node`].
    ///
    /// [`Node`]: type.Node.html
    pub fn root(&self) -> Node {
        self.root.clone()
    }

    /// Returns the first child with `svg` tag name of the root [`Node`].
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
    /// let doc = Document::from_str(
    ///     "<!--comment--><svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
    ///
    /// assert_eq!(doc.svg_element().unwrap().is_tag_name(ElementId::Svg), true);
    /// ```
    ///
    /// [`Node`]: type.Node.html
    pub fn svg_element(&self) -> Option<Node> {
        for (id, n) in self.root().children().svg() {
            if id == ElementId::Svg {
                return Some(n.clone());
            }
        }

        None
    }

    /// Removes this node and all it children from the tree.
    ///
    /// Same as `detach()`, but also removes all linked attributes from the tree.
    ///
    /// # Panics
    ///
    /// Panics if the node or one of its adjoining nodes or any children node is currently borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::{Document, ElementId, AttributeId};
    ///
    /// let mut doc = Document::from_str(
    /// "<svg xmlns='http://www.w3.org/2000/svg'>
    ///     <rect id='rect1'/>
    ///     <use xlink:href='#rect1'/>
    /// </svg>").unwrap();
    ///
    /// let mut rect_elem = doc.root().descendants().filter(|n| *n.id() == "rect1").next().unwrap();
    /// let use_elem = doc.root().descendants().filter(|n| n.is_tag_name(ElementId::Use)).next().unwrap();
    ///
    /// assert_eq!(use_elem.has_attribute(AttributeId::Href), true);
    ///
    /// // The 'remove' method will remove 'rect' element and all it's children.
    /// // Also it will remove all links to this element and it's children,
    /// // so 'use' element will no longer have the 'xlink:href' attribute.
    /// doc.remove_node(rect_elem);
    ///
    /// assert_eq!(use_elem.has_attribute(AttributeId::Href), false);
    /// ```
    pub fn remove_node(&mut self, node: Node) {
        let mut ids = Vec::with_capacity(16);
        self._remove(node.clone(), &mut ids);
    }

    fn _remove(&mut self, mut node: Node, ids: &mut Vec<AttributeQName>) {
        ids.clear();

        for (_, attr) in node.attributes().iter().svg() {
            match attr.value {
                  AttributeValue::Link(_)
                | AttributeValue::FuncLink(_)
                | AttributeValue::Paint(_, _) => {
                    ids.push(attr.name.clone())
                }
                _ => {}
            }
        }

        for name in ids.iter() {
            node.remove_attribute(name.as_ref());
        }

        // remove all attributes that linked to this node
        let linked_nodes = node.linked_nodes().clone();
        for mut linked in linked_nodes {
            ids.clear();

            for (_, attr) in linked.attributes().iter().svg() {
                match attr.value {
                      AttributeValue::Link(ref link)
                    | AttributeValue::FuncLink(ref link)
                    | AttributeValue::Paint(ref link, _) => {
                        if *link == node {
                            ids.push(attr.name.clone())
                        }
                    }
                    _ => {}
                }
            }

            for name in ids.iter() {
                linked.remove_attribute(name.as_ref());
            }
        }


        // repeat for children
        for child in node.children() {
            self._remove(child, ids);
        }

        node.detach();
        let key = node.borrow_mut().storage_key.take();
        assert!(key.is_some(), "node was already removed");
        self.storage.remove(key.unwrap());
    }

    // TODO: maybe rename to retain to match Attributes::retain
    /// Removes only the children nodes specified by the predicate.
    ///
    /// Uses [remove()](#method.remove), not [detach()](#method.detach) internally.
    ///
    /// The `root` node will be ignored.
    pub fn drain<P>(&mut self, root: Node, f: P) -> usize
        where P: Fn(&Node) -> bool
    {
        let mut count = 0;
        self._drain(root, &f, &mut count);
        count
    }

    fn _drain<P>(&mut self, parent: Node, f: &P, count: &mut usize)
        where P: Fn(&Node) -> bool
    {
        let mut node = parent.first_child();
        while let Some(n) = node {
            if f(&n) {
                node = n.next_sibling();
                self.remove_node(n);
                *count += 1;
            } else {
                if n.has_children() {
                    self._drain(n.clone(), f, count);
                }

                node = n.next_sibling();
            }
        }
    }

    /// Returns a copy of a current node without children.
    ///
    /// All attributes except `id` will be copied, because `id` must be unique.
    pub fn copy_node(&mut self, node: Node) -> Node {
        match node.node_type() {
            NodeType::Element => {
                let mut elem = self.create_element(node.tag_name().as_ref());

                for attr in node.attributes().iter() {
                    elem.set_attribute(attr.clone());
                }

                elem
            }
            _ => {
                self.create_node(node.node_type(), node.text().clone())
            }
        }
    }

    /// Returns a deep copy of a current node with all it's children.
    ///
    /// All attributes except `id` will be copied, because `id` must be unique.
    pub fn copy_node_deep(&mut self, node: Node) -> Node {
        let mut root = self.copy_node(node.clone());
        self._make_deep_copy(&mut root, &node);
        root
    }

    fn _make_deep_copy(&mut self, parent: &mut Node, node: &Node) {
        for child in node.children() {
            let mut new_node = self.copy_node(child.clone());
            parent.append(new_node.clone());

            if child.has_children() {
                self._make_deep_copy(&mut new_node, &child);
            }
        }
    }
}

impl WriteBuffer for Document {
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>) {
        writer::write_dom(self, opt, buf);
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        for (_, node) in self.storage.iter_mut() {
            node.attributes_mut().clear();
            node.linked_nodes_mut().clear();
            node.detach();
        }
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.with_write_opt(&WriteOptions::default()))
    }
}
