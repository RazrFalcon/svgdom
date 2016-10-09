// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::cell::{RefCell, Ref, RefMut};
use std::fmt;
use std::rc::{Rc, Weak};
use std::str::FromStr;

use attribute::*;
use parser::parse_svg;
use write;
use super::{
    AttributeId,
    Attributes,
    ElementId,
    Error,
    ParseOptions,
    WriteBuffer,
    WriteOptions,
    WriteToString,
};

/// Container of [`Node`](struct.Node.html)s.
pub struct Document {
    root: Node,
}

impl Document {
    /// Constructs a new `Document`.
    pub fn new() -> Document {
        Document {
            root: Document::new_node(None, NodeType::Root, None, None)
        }
    }

    /// Constructs a new `Document` from the `data` using a default `ParseOptions`.
    pub fn from_data(data: &[u8]) -> Result<Document, Error> {
        Document::from_data_with_opt(data, &ParseOptions::default())
    }

    /// Constructs a new `Document` from the `data` using a supplied `ParseOptions`.
    pub fn from_data_with_opt(data: &[u8], opt: &ParseOptions) -> Result<Document, Error> {
        parse_svg(data, opt)
    }

    /// Constructs a new `Node` with `Element` type.
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    pub fn create_element(&self, eid: ElementId) -> Node {
        Document::new_node(Some(self.root.0.clone()), NodeType::Element,
                           Some(TagName::Id(eid)), None)
    }

    /// Constructs a new `Node` with `Element` type and non-SVG tag name.
    ///
    /// Constructed node do belong to this document, but not added to it tree structure.
    pub fn create_nonsvg_element(&self, tag_name: &str) -> Result<Node, Error> {
        if tag_name.is_empty() {
            return Err(Error::EmptyTagName);
        }

        Ok(Document::new_node(Some(self.root.0.clone()), NodeType::Element,
                              Some(TagName::Name(tag_name.to_owned())), None))
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
    /// assert_eq!(doc.svg_element().unwrap().is_tag_id(ElementId::Svg), true);
    /// ```
    pub fn svg_element(&self) -> Option<Node> {
        for n in self.root.children() {
            if n.is_tag_id(ElementId::Svg) {
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

    /// Returns an iterator over descendant SVG elements.
    pub fn descendants(&self) -> Descendants {
        self.root.descendants()
    }

    /// Returns an iterator over descendant SVG nodes.
    pub fn descendant_nodes(&self) -> DescendantNodes {
        self.root.descendant_nodes()
    }

    /// Returns an iterator to this node's children elements.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn children(&self) -> Children {
        self.root.children()
    }

    fn new_node(doc: Option<Link>, node_type: NodeType, tag_name: Option<TagName>,
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
            tag_name: tag_name,
            id: String::new(),
            attributes: Attributes::new(),
            ext_attributes: HashMap::new(),
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
        write::write_dom(self, opt, buf);
    }
}

impl_display!(Document);

macro_rules! try_opt {
    ($expr: expr) => {
        match $expr {
            Some(value) => value,
            None => return None
        }
    }
}

/// Representation of the SVG node.
///
/// This is the main block of the library.
///
/// It's designed as classical DOM node. We have links to a parent node, first child, last child,
/// previous sibling and next sibling. So DOM nodes manipulations are very fast.
///
/// Node consists of:
///  - The `NodeType`, which indicates it's type. It can't be changed.
///  - Optional `TagName`, used only by element nodes.
///  - Unique ID of the element node. Can be set to nodes with other types,
///    but without any affect.
///  - List of SVG attributes.
///  - List of unknown attributes.
///  - Optional text data, which is used by non-element nodes.
///
/// Most of the API are designed to work with SVG elements and attributes.
/// Processing of non-SVG data is pretty hard/verbose, since it's an SVG DOM, not an XML.
pub struct Node(Rc<RefCell<NodeData>>);

impl Node {
    /// Returns a `Document` that owns this node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn document(&self) -> Document {
        // TODO: will fail on root node
        Document { root: Node(self.0.borrow().doc.as_ref().unwrap().clone()) }
    }

    /// Returns a parent node, unless this node is the root of the tree.
    ///
    /// This method also returns `NodeType::Root`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn parent(&self) -> Option<Node> {
        // TODO: we actually always have a parent - Root node
        Some(Node(try_opt!(try_opt!(self.0.borrow().parent.as_ref()).upgrade())))
    }

    /// Returns a parent element with selected tag name.
    ///
    /// Returns `None` if this node is the root of the tree or there is no parent
    /// nodes with such tag name.
    ///
    /// # Panics
    ///
    /// Panics if any of the parent nodes is currently mutability borrowed.
    pub fn parent_element<T>(&self, tag_name: T) -> Option<Node>
        where TagName: From<T>
    {
        let tg = TagName::from(tag_name);
        let mut parent = self.parent();
        while let Some(p) = parent {
            if p.is_tag_name(&tg) {
                return Some(p.clone());
            }
            parent = p.parent();
        }
        None
    }

    /// Returns `true` if the node has a parent node.
    ///
    /// This method ignores root node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_data(
    /// b"<svg>
    ///     <rect/>
    /// </svg>").unwrap();
    ///
    /// let svg = doc.first_child().unwrap();
    /// let rect = svg.first_child().unwrap();
    /// assert_eq!(svg.has_parent(), false);
    /// assert_eq!(rect.has_parent(), true);
    /// ```
    pub fn has_parent(&self) -> bool {
        match self.parent() {
            Some(node) => {
                match node.node_type() {
                    NodeType::Root => false,
                    _ => true,
                }
            }
            None => false,
        }
    }

    /// Returns an iterator to this node's children elements.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn children(&self) -> Children {
        if let Some(mut child) = self.first_child() {
            loop {
                if child.is_svg_element() {
                    return Children(Some(child));
                }

                if let Some(c) = child.next_sibling() {
                    child = c;
                } else {
                    break;
                }
            }
        }

        Children(None)
    }

    /// Returns `true` is this node has children elements.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_children(&self) -> bool {
        // we don't used self.children iterator for performance reasons
        if let Some(mut child) = self.first_child() {
            loop {
                if child.is_svg_element() {
                    return true;
                }

                if let Some(c) = child.next_sibling() {
                    child = c;
                } else {
                    break;
                }
            }
        }

        false
    }

    /// Returns an iterator to this node's children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn children_nodes(&self) -> ChildrenNodes {
        ChildrenNodes(self.first_child())
    }

    /// Returns `true` is this node has children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_children_nodes(&self) -> bool {
        self.first_child().is_some()
    }

    /// Returns the first child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn first_child(&self) -> Option<Node> {
        Some(Node(try_opt!(self.0.borrow().first_child.as_ref()).clone()))
    }

    /// Returns the last child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn last_child(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().last_child.as_ref()).upgrade())))
    }

    /// Returns the previous sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn previous_sibling(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().previous_sibling.as_ref()).upgrade())))
    }

    /// Returns the previous sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn next_sibling(&self) -> Option<Node> {
        Some(Node(try_opt!(self.0.borrow().next_sibling.as_ref()).clone()))
    }

    /// Returns `true` if two `Node`s are the same node.
    pub fn same_node(&self, other: &Node) -> bool {
        same_rc(&self.0, &other.0)
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    ///
    /// # Panics
    ///
    /// Panics if the node or one of its adjoining nodes is currently borrowed.
    pub fn detach(&self) {
        self.0.borrow_mut().detach();
    }

    /// Removes this node and all it children from the tree.
    ///
    /// Same as `detach()`, but also unlinks all linked nodes and attributes.
    ///
    /// # Panics
    ///
    /// Panics if the node or one of its adjoining nodes or any children node is currently borrowed.
    pub fn remove(&self) {
        Node::_remove(self);
        self.detach();
    }

    fn _remove(node: &Node) {
        // remove link attributes, which will trigger nodes unlink
        let mut ids: Vec<AttributeId> = node.attributes().iter()
                                        .filter(|a| a.is_link() || a.is_func_link())
                                        .map(|a| a.id)
                                        .collect();
        for id in &ids {
            node.remove_attribute(*id);
        }

        // remove all attributes that linked to this node
        for linked in node.linked_nodes() {
            ids.clear();

            for attr in linked.attributes().iter() {
                match attr.value {
                    AttributeValue::Link(ref link) | AttributeValue::FuncLink(ref link) => {
                        if link == node {
                            ids.push(attr.id);
                        }
                    }
                    _ => {}
                }
            }

            for id in &ids {
                linked.remove_attribute(*id);
            }
        }

        // repeat for children
        for child in node.children() {
            Node::_remove(&child);
        }
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new child, or one of their adjoining nodes is currently borrowed.
    pub fn append(&self, new_child: &Node) {
        let mut self_borrow = self.0.borrow_mut();
        let mut last_child_opt = None;
        let nc = new_child.clone();
        {
            let mut new_child_borrow = nc.0.borrow_mut();
            new_child_borrow.detach();
            new_child_borrow.parent = Some(Rc::downgrade(&self.0));
            if let Some(last_child_weak) = self_borrow.last_child.take() {
                if let Some(last_child_strong) = last_child_weak.upgrade() {
                    new_child_borrow.previous_sibling = Some(last_child_weak);
                    last_child_opt = Some(last_child_strong);
                }
            }
            self_borrow.last_child = Some(Rc::downgrade(&nc.0));
        }

        if let Some(last_child_strong) = last_child_opt {
            let mut last_child_borrow = last_child_strong.borrow_mut();
            debug_assert!(last_child_borrow.next_sibling.is_none());
            last_child_borrow.next_sibling = Some(nc.0);
        } else {
            // No last child
            debug_assert!(self_borrow.first_child.is_none());
            self_borrow.first_child = Some(nc.0);
        }
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new child, or one of their adjoining nodes is currently borrowed.
    pub fn prepend(&self, new_child: Node) {
        let mut self_borrow = self.0.borrow_mut();
        {
            let mut new_child_borrow = new_child.0.borrow_mut();
            new_child_borrow.detach();
            new_child_borrow.parent = Some(Rc::downgrade(&self.0));
            match self_borrow.first_child.take() {
                Some(first_child_strong) => {
                    {
                        let mut first_child_borrow = first_child_strong.borrow_mut();
                        debug_assert!(first_child_borrow.previous_sibling.is_none());
                        first_child_borrow.previous_sibling = Some(Rc::downgrade(&new_child.0));
                    }
                    new_child_borrow.next_sibling = Some(first_child_strong);
                }
                None => {
                    debug_assert!(self_borrow.first_child.is_none());
                    self_borrow.last_child = Some(Rc::downgrade(&new_child.0));
                }
            }
        }
        self_borrow.first_child = Some(new_child.0);
    }

    /// Insert a new sibling after this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_after(&self, new_sibling: Node) {
        // TODO: add an example, since we need to detach 'new_sibling'
        //       before passing it to this method
        let mut self_borrow = self.0.borrow_mut();
        {
            let mut new_sibling_borrow = new_sibling.0.borrow_mut();
            new_sibling_borrow.detach();
            new_sibling_borrow.parent = self_borrow.parent.clone();
            new_sibling_borrow.previous_sibling = Some(Rc::downgrade(&self.0));
            match self_borrow.next_sibling.take() {
                Some(next_sibling_strong) => {
                    {
                        let mut next_sibling_borrow = next_sibling_strong.borrow_mut();
                        debug_assert!({
                            let weak = next_sibling_borrow.previous_sibling.as_ref().unwrap();
                            same_rc(&weak.upgrade().unwrap(), &self.0)
                        });
                        next_sibling_borrow.previous_sibling = Some(Rc::downgrade(&new_sibling.0));
                    }
                    new_sibling_borrow.next_sibling = Some(next_sibling_strong);
                }
                None => {
                    if let Some(parent_ref) = self_borrow.parent.as_ref() {
                        if let Some(parent_strong) = parent_ref.upgrade() {
                            let mut parent_borrow = parent_strong.borrow_mut();
                            parent_borrow.last_child = Some(Rc::downgrade(&new_sibling.0));
                        }
                    }
                }
            }
        }
        self_borrow.next_sibling = Some(new_sibling.0);
    }

    /// Insert a new sibling before this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_before(&self, new_sibling: Node) {
        let mut self_borrow = self.0.borrow_mut();
        let mut previous_sibling_opt = None;
        {
            let mut new_sibling_borrow = new_sibling.0.borrow_mut();
            new_sibling_borrow.detach();
            new_sibling_borrow.parent = self_borrow.parent.clone();
            new_sibling_borrow.next_sibling = Some(self.0.clone());
            if let Some(previous_sibling_weak) = self_borrow.previous_sibling.take() {
                if let Some(previous_sibling_strong) = previous_sibling_weak.upgrade() {
                    new_sibling_borrow.previous_sibling = Some(previous_sibling_weak);
                    previous_sibling_opt = Some(previous_sibling_strong);
                }
            }
            self_borrow.previous_sibling = Some(Rc::downgrade(&new_sibling.0));
        }

        if let Some(previous_sibling_strong) = previous_sibling_opt {
            let mut previous_sibling_borrow = previous_sibling_strong.borrow_mut();
            debug_assert!({
                let rc = previous_sibling_borrow.next_sibling.as_ref().unwrap();
                same_rc(rc, &self.0)
            });
            previous_sibling_borrow.next_sibling = Some(new_sibling.0);
        } else {
            // No previous sibling.
            if let Some(parent_ref) = self_borrow.parent.as_ref() {
                if let Some(parent_strong) = parent_ref.upgrade() {
                    let mut parent_borrow = parent_strong.borrow_mut();
                    parent_borrow.first_child = Some(new_sibling.0);
                }
            }
        }
    }

    /// Returns node's type.
    ///
    /// You can't change the type of the node. Only create a new one.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn node_type(&self) -> NodeType {
        self.0.borrow().node_type
    }

    /// Returns a text data of the node, if there are any.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn text(&self) -> Option<Ref<String>> {
        let b = self.0.borrow();
        match b.text {
            Some(_) => Some(Ref::map(self.0.borrow(), |n| n.text.as_ref().unwrap())),
            None => None,
        }
    }

    /// Sets a text data to the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn set_text(&self, text: &str) {
        debug_assert!(self.node_type() != NodeType::Element);
        let mut b = self.0.borrow_mut();
        b.text = Some(text.to_owned());
    }

    /// Returns `true` if there are any children text nodes.
    ///
    /// This method is recursive.
    ///
    /// # Panics
    ///
    /// Panics if the node or any descendants nodes are currently mutability borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_data(
    /// b"<svg>
    ///     <g>
    ///         <text>Some text</text>
    ///     </g>
    ///     <rect/>
    /// </svg>").unwrap();
    ///
    /// let svg = doc.first_child().unwrap();
    /// let g = svg.first_child().unwrap();
    /// assert_eq!(g.has_text_children(), true);
    ///
    /// let text = g.first_child().unwrap();
    /// assert_eq!(text.has_text_children(), true);
    ///
    /// let rect = g.next_sibling().unwrap();
    /// assert_eq!(rect.has_text_children(), false);
    /// ```
    pub fn has_text_children(&self) -> bool {
        // TODO: to has_text_child and check only first child

        for node in self.descendant_nodes() {
            if node.node_type() == NodeType::Text {
                return true;
            }
        }
        false
    }

    /// Sets an ID of the element.
    ///
    /// Only element nodes can contain an ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_id<S: Into<String>>(&self, id: S) {
        // TODO: check that it's unique.
        debug_assert!(self.node_type() == NodeType::Element);
        let mut self_borrow = self.0.borrow_mut();
        self_borrow.id = id.into();
    }

    /// Returns an ID of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn id(&self) -> Ref<String> {
        Ref::map(self.0.borrow(), |n| &n.id)
    }

    /// Returns `true` if node has a not empty ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_id(&self) -> bool {
        !self.0.borrow().id.is_empty()
    }

    /// Returns `true` if node has an `Element` type and an SVG tag name.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn is_svg_element(&self) -> bool {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref tag) => {
                match *tag {
                    TagName::Id(_) => true,
                    TagName::Name(_) => false,
                }
            }
            None => false,
        }
    }

    /// Sets a tag id of the element node.
    ///
    /// Only element nodes can contain tag name.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_tag_id(&self, eid: ElementId) {
        debug_assert!(self.node_type() == NodeType::Element);

        let mut self_borrow = self.0.borrow_mut();
        self_borrow.tag_name = Some(TagName::Id(eid));
    }

    /// Sets a tag name of the element node.
    ///
    /// Only element nodes can contain tag name.
    ///
    /// # Errors
    ///
    /// The string tag name must be non-empty.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_tag_name(&self, tag_name: &str) -> Result<(), Error> {
        debug_assert!(self.node_type() == NodeType::Element);

        if tag_name.is_empty() {
            return Err(Error::EmptyTagName);
        }

        let mut self_borrow = self.0.borrow_mut();
        self_borrow.tag_name = Some(TagName::Name(tag_name.to_owned()));

        Ok(())
    }

    /// Returns a tag name of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn tag_name(&self) -> Option<Ref<TagName>> {
        let b = self.0.borrow();
        match b.tag_name {
            Some(_) => Some(Ref::map(self.0.borrow(), |n| n.tag_name.as_ref().unwrap())),
            None => None,
        }
    }

    /// Returns a tag name id of the SVG element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn tag_id(&self) -> Option<ElementId> {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref t) => {
                match *t {
                    TagName::Id(ref id) => Some(*id),
                    TagName::Name(_) => None,
                }
            }
            None => None,
        }
    }

    /// Returns `true` if node has the same tag name id as supplied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn is_tag_id(&self, eid: ElementId) -> bool {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref v) => {
                match *v {
                    TagName::Id(ref id) => *id == eid,
                    TagName::Name(_) => false,
                }
            }
            None => false,
        }
    }

    /// Returns `true` if node has the same tag name as supplied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn is_tag_name(&self, tag_name: &TagName) -> bool {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref v) => v == tag_name,
            None => false,
        }
    }

    /// Returns `true` if node has a direct child with the same tag name as supplied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_child_with_tag_name(&self, tag_name: &TagName) -> bool {
        self.children().any(|n| n.is_tag_name(tag_name))
    }

    /// Inserts a new SVG attribute into attributes list.
    ///
    /// This method will overwrite an existing attribute with the same id.
    ///
    /// Use it to insert/create new attributes.
    /// For existing attributes use `Node::set_attribute_object()`.
    ///
    /// You can't use this method to set referenced attributes.
    /// Use `Node::set_link_attribute()` instead.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_attribute<T>(&self, id: AttributeId, value: T)
        where AttributeValue: From<T>
    {
        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(id);

        let a = Attribute::new(id, value);
        let mut attrs = self.attributes_mut();
        attrs.insert(a);
    }

    /// Inserts a new SVG attribute into the attributes list.
    ///
    /// This method will overwrite an existing attribute with the same id.
    ///
    /// # Panics
    ///
    /// - Panics if the node is currently borrowed.
    /// - Panics if attribute has a Link value.
    ///   Use `Node::set_link_attribute()` for such attributes.
    pub fn set_attribute_object(&self, attr: Attribute) {
        // TODO: fix stupid name
        // TODO: do not panic on invalid attribute type

        match attr.value {
            AttributeValue::Link(_) | AttributeValue::FuncLink(_) =>
                panic!("Link attributes must be set via set_link_attribute()"),
            _ => {}
        }

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(attr.id);

        let mut attrs = self.attributes_mut();
        attrs.insert(attr);
    }

    /// Inserts a new referenced SVG attribute into the attributes list.
    ///
    /// This method will overwrite an existing attribute with the same id.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::{Document, ValueId};
    /// use svgdom::AttributeId as AId;
    /// use svgdom::ElementId as EId;
    ///
    /// // Create a simple document.
    /// let doc = Document::new();
    /// let gradient = doc.create_element(EId::LinearGradient);
    /// let rect = doc.create_element(EId::Rect);
    ///
    /// doc.append(&gradient);
    /// doc.append(&rect);
    ///
    /// gradient.set_id("lg1");
    /// rect.set_id("rect1");
    ///
    /// // Set a `fill` attribute value to the `none`.
    /// // For now everything like in any other XML DOM library.
    /// rect.set_attribute(AId::Fill, ValueId::None);
    ///
    /// // Now we want to fill our rect with a gradient.
    /// // To do this we need to set a link attribute:
    /// rect.set_link_attribute(AId::Fill, gradient.clone()).unwrap();
    ///
    /// // Now our fill attribute has a link to the `gradient` node.
    /// // Not as text, aka `url(#lg1)`, but an actual reference.
    ///
    /// // This adds support for fast checking that the element is used. Which is very useful.
    ///
    /// // `gradient` is now used, since we link it.
    /// assert_eq!(gradient.is_used(), true);
    /// // Also, we can check how many elements are uses this `gradient`.
    /// assert_eq!(gradient.uses_count(), 1);
    /// // And even get this elements:
    /// assert_eq!(gradient.linked_nodes().next().unwrap(), rect);
    ///
    /// // `rect` is unused, because no one has referenced attribute that has link to it.
    /// assert_eq!(rect.is_used(), false);
    ///
    /// // Now, if we set other attribute value, `gradient` will be automatically unlinked.
    /// rect.set_attribute(AId::Fill, ValueId::None);
    /// // No one uses it anymore.
    /// assert_eq!(gradient.is_used(), false);
    /// ```
    pub fn set_link_attribute(&self, id: AttributeId, node: Node) -> Result<(), Error> {
        // TODO: rewrite to template specialization when it will be available
        // TODO: check that node is element

        if node.id().is_empty() {
            return Err(Error::ElementMustHaveAnId);
        }

        // check for recursion
        if *self.id() == *node.id() {
            return Err(Error::ElementCrosslink);
        }

        // check for recursion 2
        {
            let self_borrow = self.0.borrow();
            let v = &self_borrow.linked_nodes;

            if v.iter().any(|n| Node(n.upgrade().unwrap()) == node) {
                return Err(Error::ElementCrosslink);
            }
        }

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(id);

        {
            let a = if id == AttributeId::XlinkHref {
                Attribute::new(id, AttributeValue::Link(node.clone()))
            } else {
                Attribute::new(id, AttributeValue::FuncLink(node.clone()))
            };

            let mut attributes = self.attributes_mut();
            attributes.insert(a);
        }

        {
            let mut value_borrow = node.0.borrow_mut();
            value_borrow.linked_nodes.push(Rc::downgrade(&self.0));
        }

        Ok(())
    }

    /// Returns an iterator over linked nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn linked_nodes(&self) -> LinkAttributes {
        let self_borrow = self.0.borrow();

        LinkAttributes {
            data: self_borrow.linked_nodes.clone(),
            idx: 0,
        }
    }

    /// Returns a copy of the attribute value by `id`.
    ///
    /// Use it only for simple `AttributeValue` types, and not for `String` and `Path`,
    /// since their copying will be very expensive.
    ///
    /// Prefer `Node::attributes()`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn attribute_value(&self, id: AttributeId) -> Option<AttributeValue> {
        self.attributes().get_value(id).cloned()
    }

    /// Returns a copy of the attribute by `id`.
    ///
    /// Use it only for attributes with simple `AttributeValue` types,
    /// and not for `String` and `Path`, since their copying will be very expensive.
    ///
    /// Prefer `Node::attributes()`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn attribute(&self, id: AttributeId) -> Option<Attribute> {
        self.attributes().get(id).cloned()
    }

    /// Returns a reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn attributes(&self) -> Ref<Attributes> {
        Ref::map(self.0.borrow(), |n| &n.attributes)
    }

    /// Returns a mutable reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn attributes_mut(&self) -> RefMut<Attributes> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.attributes)
    }

    /// Returns first occurrence of the selected `AttributeId` from it's parents.
    ///
    /// This function will check all parents, not only direct one.
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::{Document, TagName, ElementId, AttributeId, Attribute};
    /// use svgdom::types::Color;
    ///
    /// let doc = Document::from_data(
    /// b"<svg stroke='blue'>
    ///     <g fill='red'>
    ///         <rect/>
    ///     </g>
    /// </svg>").unwrap();
    ///
    /// let rect = doc.first_child().unwrap().child_by_tag_name(&TagName::Id(ElementId::Rect)).unwrap();
    /// assert_eq!(rect.parent_attribute(AttributeId::Fill).unwrap(),
    ///            Attribute::new(AttributeId::Fill, Color::new(255, 0, 0)));
    /// assert_eq!(rect.parent_attribute(AttributeId::Stroke).unwrap(),
    ///            Attribute::new(AttributeId::Stroke, Color::new(0, 0, 255)));
    /// assert_eq!(rect.parent_attribute(AttributeId::Filter).is_some(), false);
    /// ```
    pub fn parent_attribute(&self, id: AttributeId) -> Option<Attribute> {
        let mut parent = self.parent();
        while let Some(p) = parent {
            if p.has_attribute(id) {
                return p.attribute(id);
            }
            parent = p.parent();
        }
        None
    }

    /// Returns `true` if the node has an attribute with such `id`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    #[inline]
    pub fn has_attribute(&self, id: AttributeId) -> bool {
        self.0.borrow().attributes.contains(id)
    }

    /// Returns `true` if the node has an attribute with such `id` and this attribute is visible.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_visible_attribute(&self, id: AttributeId) -> bool {
        self.has_attribute(id) && self.attributes().get(id).unwrap().visible
    }

    /// Returns `true` if the node has any of provided attributes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_attributes(&self, ids: &[AttributeId]) -> bool {
        let attrs = self.attributes();
        for id in ids {
            if attrs.contains(*id) {
                return true;
            }
        }

        false
    }

    /// Returns `true` if node has an attribute with such `id` and such `value`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_attribute_with_value<T>(&self, id: AttributeId, value: T) -> bool
        where AttributeValue: From<T>
    {
        match self.attribute_value(id) {
            Some(a) => a == AttributeValue::from(value),
            None => false,
        }
    }

    /// Removes an attribute from the node.
    ///
    /// It will also unlink it, if it was an referenced attribute.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attribute(&self, id: AttributeId) {
        if !self.has_attribute(id) {
            return;
        }

        let mut attrs = self.attributes_mut();

        // we must unlink referenced attributes
        if let Some(value) = attrs.get_value(id) {
            match *value {
                AttributeValue::Link(ref node) | AttributeValue::FuncLink(ref node) => {
                    let mut self_borrow = node.0.borrow_mut();
                    let ln = &mut self_borrow.linked_nodes;
                    // this code can't panic, because we know that such node exist
                    let index = ln.iter().position(|x| {
                        same_rc(&x.upgrade().unwrap(), &self.0)
                    }).unwrap();
                    ln.remove(index);
                }
                _ => {}
            }
        }

        attrs.remove(id);
    }

    /// Removes attributes from the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attributes(&self, ids: &[AttributeId]) {
        for id in ids {
            self.remove_attribute(*id);
        }
    }

    /// Returns a reference to an unknown attributes object.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn unknown_attributes(&self) -> Ref<HashMap<String,String>> {
        Ref::map(self.0.borrow(), |n| &n.ext_attributes)
    }

    /// Returns a mutable reference to an unknown attributes object.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn unknown_attributes_mut(&self) -> RefMut<HashMap<String,String>> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.ext_attributes)
    }

    /// Returns `true` if the current node is linked to any of the DOM nodes.
    ///
    /// See `Node::set_link_attribute()` for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn is_used(&self) -> bool {
        let self_borrow = self.0.borrow();
        !self_borrow.linked_nodes.is_empty()
    }

    /// Returns a number of nodes, which is linked to this node.
    ///
    /// See `Node::set_link_attribute()` for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn uses_count(&self) -> usize {
        let self_borrow = self.0.borrow();
        self_borrow.linked_nodes.len()
    }

    /// Returns true if the current node is referenced.
    ///
    /// Referenced elements are elements that do not render by itself,
    /// rather defines rendering properties for other.
    ///
    /// List: `altGlyphDef`, `clipPath`, `cursor`, `filter`, `linearGradient`, `marker`,
    /// `mask`, `pattern`, `radialGradient` and `symbol`.
    ///
    /// Details: https://www.w3.org/TR/SVG/struct.html#Head
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_data(b"<svg><linearGradient/></svg>").unwrap();
    /// let mut iter = doc.descendants();
    /// assert_eq!(iter.next().unwrap().is_referenced(), false); // svg
    /// assert_eq!(iter.next().unwrap().is_referenced(), true); // linearGradient
    /// ```
    pub fn is_referenced(&self) -> bool {
        if let Some(id) = self.tag_id() {
            match id {
                  ElementId::AltGlyphDef
                | ElementId::ClipPath
                | ElementId::Cursor
                | ElementId::Filter
                | ElementId::LinearGradient
                | ElementId::Marker
                | ElementId::Mask
                | ElementId::Pattern
                | ElementId::RadialGradient
                | ElementId::Symbol => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Returns true if the current node is a basic shape.
    ///
    /// List: `rect`, `circle`, `ellipse`, `line`, `polyline`, `polygon`.
    ///
    /// Details: https://www.w3.org/TR/SVG/shapes.html
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_data(b"<svg><rect/></svg>").unwrap();
    /// let mut iter = doc.descendants();
    /// assert_eq!(iter.next().unwrap().is_basic_shape(), false); // svg
    /// assert_eq!(iter.next().unwrap().is_basic_shape(), true); // rect
    /// ```
    pub fn is_basic_shape(&self) -> bool {
        if let Some(id) = self.tag_id() {
            match id {
                  ElementId::Rect
                | ElementId::Circle
                | ElementId::Ellipse
                | ElementId::Line
                | ElementId::Polyline
                | ElementId::Polygon => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Returns true if the current node is a container element.
    ///
    /// List: `a`, `defs`, `glyph`, `g`, `marker`, `mask`, `missing-glyph`, `pattern`, `svg`,
    /// `switch` and `symbol`.
    ///
    /// Details: https://www.w3.org/TR/SVG/intro.html#TermContainerElement
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_data(b"<svg><rect/></svg>").unwrap();
    /// let mut iter = doc.descendants();
    /// assert_eq!(iter.next().unwrap().is_container(), true); // svg
    /// assert_eq!(iter.next().unwrap().is_container(), false); // rect
    /// ```
    pub fn is_container(&self) -> bool {
        if let Some(id) = self.tag_id() {
            match id {
                  ElementId::A
                | ElementId::Defs
                | ElementId::Glyph
                | ElementId::G
                | ElementId::Marker
                | ElementId::Mask
                | ElementId::MissingGlyph
                | ElementId::Pattern
                | ElementId::Svg
                | ElementId::Switch
                | ElementId::Symbol => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Returns a child `Node` with selected `TagName`.
    ///
    /// This function is recursive. Current node excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::{Document, TagName, ElementId};
    ///
    /// let doc = Document::from_data(
    /// b"<svg>
    ///     <g>
    ///         <rect/>
    ///     </g>
    ///     <myelem/>
    /// </svg>").unwrap();
    ///
    /// let svg = doc.first_child().unwrap();
    /// // current node will be skipped
    /// assert_eq!(svg.child_by_tag_name(&TagName::Id(ElementId::Svg)).is_some(), false);
    /// // we'll get true since current method is recursive
    /// assert_eq!(svg.child_by_tag_name(&TagName::Id(ElementId::Rect)).is_some(), true);
    /// // check for not existing element
    /// assert_eq!(svg.child_by_tag_name(&TagName::Id(ElementId::Path)).is_some(), false);
    /// // check for non-svg element
    /// assert_eq!(svg.child_by_tag_name(&TagName::from("myelem")).is_some(), true);
    /// ```
    pub fn child_by_tag_name(&self, tag_name: &TagName) -> Option<Node> {
        let iter = self.descendant_nodes().skip(1);
        for node in iter {
            if node.is_tag_name(tag_name) {
                return Some(node.clone());
            }
        }
        None
    }

    /// Returns `Node` if the current node contains child with selected `ElementId`.
    ///
    /// Shorthand for `Node::child_by_tag_name(&TagName::Id(id))`.
    pub fn child_by_tag_id(&self, id: ElementId) -> Option<Node> {
        self.child_by_tag_name(&TagName::Id(id))
    }

    /// Returns an iterator over descendant elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use svgdom::{Document, ElementId};
    ///
    /// let doc = Document::from_data(
    /// b"<!--comment-->
    /// <svg>
    ///   <g>
    ///     <nonsvg/>
    ///     <rect/>
    ///   </g>
    ///   <text>Text</text>
    ///   <nonsvg/>
    /// </svg>").unwrap();
    ///
    /// let mut iter = doc.descendants();
    /// assert_eq!(iter.next().unwrap().is_tag_id(ElementId::Svg), true);
    /// assert_eq!(iter.next().unwrap().is_tag_id(ElementId::G), true);
    /// assert_eq!(iter.next().unwrap().is_tag_id(ElementId::Rect), true);
    /// assert_eq!(iter.next().unwrap().is_tag_id(ElementId::Text), true);
    /// assert_eq!(iter.next().is_none(), true);
    /// ```
    pub fn descendants(&self) -> Descendants {
        Descendants(self.traverse())
    }

    /// Returns an iterator over descendant nodes.
    pub fn descendant_nodes(&self) -> DescendantNodes {
        DescendantNodes(self.traverse())
    }

    /// Returns an iterator over descendant nodes.
    ///
    /// More complex alternative of the `Node::descendant_nodes()`.
    pub fn traverse(&self) -> Traverse {
        Traverse {
            root: self.clone(),
            next: Some(NodeEdge::Start(self.clone())),
        }
    }
}

fn same_rc<T>(a: &Rc<T>, b: &Rc<T>) -> bool {
    let a: *const T = &**a;
    let b: *const T = &**b;
    a == b
}

/// Cloning a `Node` only increments a reference count. It does not copy the data.
impl Clone for Node {
    fn clone(&self) -> Node {
        Node(self.0.clone())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.same_node(other)
    }
}

// TODO: write better messages
impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type() {
            NodeType::Root => write!(f, "Root node"),
            NodeType::Element => write!(f, "<{:?} id={:?}>", self.tag_name().unwrap(), self.id()),
            NodeType::Declaration => write!(f, "Declaration node"),
            NodeType::Comment => write!(f, "Comment node"),
            NodeType::Cdata => write!(f, "CDATA node"),
            NodeType::Text => write!(f, "Text node"),
        }
    }
}

/// List of supported node types.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum NodeType {
    /// Root node of the `Document`.
    ///
    /// Constructed with `Document`. Unavailable to user.
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

type Link = Rc<RefCell<NodeData>>;
type WeakLink = Weak<RefCell<NodeData>>;

#[allow(dead_code)]
struct NodeData {
    // TODO: check that doc is equal in append, insert, etc.
    doc: Option<Link>,

    parent: Option<WeakLink>,
    first_child: Option<Link>,
    last_child: Option<WeakLink>,
    previous_sibling: Option<WeakLink>,
    next_sibling: Option<Link>,

    node_type: NodeType, // TODO: should be immutable/const somehow
    tag_name: Option<TagName>,
    id: String,
    attributes: Attributes,
    ext_attributes: HashMap<String,String>,
    linked_nodes: Vec<WeakLink>,
    text: Option<String>,
}

impl NodeData {
    /// Detach a node from its parent and siblings. Children are not affected.
    fn detach(&mut self) {
        let parent_weak = self.parent.take();
        let previous_sibling_weak = self.previous_sibling.take();
        let next_sibling_strong = self.next_sibling.take();

        let previous_sibling_opt = previous_sibling_weak.as_ref().and_then(|weak| weak.upgrade());

        if let Some(next_sibling_ref) = next_sibling_strong.as_ref() {
            let mut next_sibling_borrow = next_sibling_ref.borrow_mut();
            next_sibling_borrow.previous_sibling = previous_sibling_weak;
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                let mut parent_borrow = parent_strong.borrow_mut();
                parent_borrow.last_child = previous_sibling_weak;
            }
        }

        if let Some(previous_sibling_strong) = previous_sibling_opt {
            let mut previous_sibling_borrow = previous_sibling_strong.borrow_mut();
            previous_sibling_borrow.next_sibling = next_sibling_strong;
        } else if let Some(parent_ref) = parent_weak.as_ref() {
            if let Some(parent_strong) = parent_ref.upgrade() {
                let mut parent_borrow = parent_strong.borrow_mut();
                parent_borrow.first_child = next_sibling_strong;
            }
        }
    }
}

/// Wrapper arrow element tag name.
#[derive(Clone,PartialEq)]
pub enum TagName {
    /// For SVG elements.
    Id(ElementId),
    /// For unknown elements.
    Name(String),
}

impl From<ElementId> for TagName {
    fn from(value: ElementId) -> TagName {
        TagName::Id(value)
    }
}

impl From<String> for TagName {
    fn from(value: String) -> TagName {
        TagName::Name(value)
    }
}

impl<'a> From<&'a str> for TagName {
    fn from(value: &str) -> TagName {
        TagName::Name(String::from_str(value).unwrap())
    }
}

impl fmt::Debug for TagName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TagName::Id(ref id) => write!(f, "{}", id.name()),
            TagName::Name(ref name) => write!(f, "{}", name),
        }
    }
}

pub struct LinkAttributes {
    data: Vec<WeakLink>,
    idx: usize,
}

impl Iterator for LinkAttributes {
    type Item = Node;

    fn next(&mut self) -> Option<Node> {
        let i = self.idx;
        self.idx += 1;

        if i < self.data.len() {
            match self.data[i].upgrade() {
                Some(n) => Some(Node(n)),
                None => None,
            }
        } else {
            None
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone)]
pub enum NodeEdge {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node`s descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(Node),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node`s descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(Node),
}


/// An iterator of `Node`s to a given node and its descendants, in tree order.
#[derive(Clone)]
pub struct Traverse {
    root: Node,
    next: Option<NodeEdge>,
}

impl Iterator for Traverse {
    type Item = NodeEdge;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutability borrowed.
    fn next(&mut self) -> Option<NodeEdge> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::Start(ref node) => {
                        match node.first_child() {
                            Some(first_child) => Some(NodeEdge::Start(first_child)),
                            None => Some(NodeEdge::End(node.clone()))
                        }
                    }
                    NodeEdge::End(ref node) => {
                        if node.same_node(&self.root) {
                            None
                        } else {
                            match node.next_sibling() {
                                Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                                None => match node.parent() {
                                    Some(parent) => Some(NodeEdge::End(parent)),

                                    // `node.parent()` here can only be `None`
                                    // if the tree has been modified during iteration,
                                    // but silently stopping iteration
                                    // seems a more sensible behavior than panicking.
                                    None => None
                                }
                            }
                        }
                    }
                };
                Some(item)
            }
            None => None
        }
    }
}

/// An iterator of `Node`s to a given node and its descendants, in tree order.
pub struct DescendantNodes(Traverse);

impl Iterator for DescendantNodes {
    type Item = Node;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutability borrowed.
    fn next(&mut self) -> Option<Node> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
}

#[derive(Clone)]
pub struct Descendants(Traverse);

impl Descendants {
    // TODO: find better way
    pub fn skip_children(&mut self) {
        // TODO: do nothing if current node does not have any children

        let n = match self.next() {
            Some(n) => n.parent().unwrap(),
            None => return,
        };

        if !n.has_children() {
            return;
        }

        loop {
            match self.0.next() {
                Some(NodeEdge::Start(_)) => {}
                Some(NodeEdge::End(node)) => {
                    if n == node {
                        break;
                    }
                }
                None => break
            }
        }
    }
}

impl Iterator for Descendants {
    type Item = Node;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutability borrowed.
    fn next(&mut self) -> Option<Node> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => {
                    if node.is_svg_element() {
                        return Some(node)
                    }
                }
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
}

macro_rules! impl_node_iterator {
    ($name: ident, $next: expr) => {
        impl Iterator for $name {
            type Item = Node;

            /// # Panics
            ///
            /// Panics if the node about to be yielded is currently mutability borrowed.
            fn next(&mut self) -> Option<Node> {
                match self.0.take() {
                    Some(node) => {
                        self.0 = $next(&node);
                        Some(node)
                    }
                    None => None
                }
            }
        }
    }
}

/// An iterator of `Node`s to the children of a given node.
pub struct Children(Option<Node>);
impl_node_iterator!(Children, |node: &Node| {
    let mut curr = node.clone();
    while let Some(n) = curr.next_sibling() {
        if n.node_type() == NodeType::Element {
            return Some(n);
        }
        curr = n.clone();
    }
    None
});

/// An iterator of `Node`s to the children of a given node.
pub struct ChildrenNodes(Option<Node>);
impl_node_iterator!(ChildrenNodes, |node: &Node| node.next_sibling());
