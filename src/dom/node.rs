// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::{
    Ref,
    RefMut
};
use std::rc::Rc;
use std::fmt;

use error::Result;
use {
    Attribute,
    AttributeId,
    AttributeNameRef,
    Attributes,
    AttributeValue,
    Children,
    Descendants,
    Document,
    ElementId,
    ErrorKind,
    LinkedNodes,
    Name,
    NameRef,
    NodeType,
    Parents,
    SvgId,
    TagName,
    TagNameRef,
    Traverse,
};
use super::node_data::{
    Link,
    NodeData,
};

macro_rules! try_opt {
    ($expr: expr) => {
        match $expr {
            Some(value) => value,
            None => return None
        }
    }
}

impl SvgId for ElementId {
    fn name(&self) -> &str { self.name() }
}


impl<'a, N, V> From<(N, V)> for Attribute
    where AttributeNameRef<'a>: From<N>, AttributeValue: From<V>
{
    fn from(v: (N, V)) -> Self {
        Attribute::new(v.0, v.1)
    }
}

impl<'a> From<(AttributeId, Node)> for Attribute {
    fn from(v: (AttributeId, Node)) -> Self {
        if v.0 == AttributeId::XlinkHref {
            Attribute::new(v.0, AttributeValue::Link(v.1))
        } else {
            Attribute::new(v.0, AttributeValue::FuncLink(v.1))
        }
    }
}


/// Representation of the SVG node.
///
/// This is the main block of the library.
///
/// It's designed as classical DOM node. It has links to a parent node, first child, last child,
/// previous sibling and next sibling. So DOM nodes manipulations are very fast.
///
/// Node consists of:
///
/// - The [`NodeType`], which indicates it's type. It can't be changed.
/// - Optional [`TagName`], used only by element nodes.
/// - Unique ID of the `Element` node. Can be set to nodes with other types,
///   but without any affect.
/// - [`Attributes`] - list of [`Attribute`]s.
/// - List of linked nodes. [Details.](#method.set_attribute_checked)
/// - Text data, which is used by non-element nodes. Empty by default.
///
/// [`Attribute`]: struct.Attribute.html
/// [`Attributes`]: struct.Attributes.html
/// [`NodeType`]: enum.NodeType.html
/// [`TagName`]: type.TagName.html
pub struct Node(pub Link);

impl Node {
    /// Returns a `Document` that owns this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node is currently mutably borrowed.
    /// - Panics if the node is a root node.
    pub fn document(&self) -> Document {
        // TODO: will fail on root node
        debug_assert_ne!(self.node_type(), NodeType::Root);
        Document { root: Node(self.0.borrow().doc.as_ref().unwrap().upgrade().unwrap()) }
    }

    /// Returns a parent node, unless this node is the root of the tree.
    ///
    /// This method also returns `NodeType::Root`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn parent(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().parent.as_ref()).upgrade())))
    }

    /// Returns `true` if the node has a parent node.
    ///
    /// This method ignores root node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    ///
    /// # Examples
    /// ```
    /// use svgdom::Document;
    ///
    /// let doc = Document::from_str(
    /// "<svg>
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
            Some(node) => node.node_type() != NodeType::Root,
            None => false,
        }
    }

    // TODO: place before has_parent
    /// Returns an iterator over node's parents.
    ///
    /// Current node is not included.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn parents(&self) -> Parents {
        Parents::new(self.parent())
    }

    /// Returns an iterator over parent nodes.
    ///
    /// Current node is included.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn parents_with_self(&self) -> Parents {
        Parents::new(Some(self.clone()))
    }

    /// Returns an iterator to this node's children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn children(&self) -> Children {
        Children::new(self.first_child())
    }

    /// Returns `true` if this node has children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_children(&self) -> bool {
        self.first_child().is_some()
    }

    // TODO: add has_single_child

    /// Returns the first child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn first_child(&self) -> Option<Node> {
        Some(Node(Rc::clone(try_opt!(self.0.borrow().first_child.as_ref()))))
    }

    /// Returns the last child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn last_child(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().last_child.as_ref()).upgrade())))
    }

    /// Returns the previous sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn previous_sibling(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().prev_sibling.as_ref()).upgrade())))
    }

    /// Returns the next sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn next_sibling(&self) -> Option<Node> {
        Some(Node(Rc::clone(try_opt!(self.0.borrow().next_sibling.as_ref()))))
    }

    /// Returns an iterator over descendant nodes.
    pub fn descendants(&self) -> Descendants {
        Descendants::new(self)
    }

    /// Returns an iterator over descendant nodes.
    ///
    /// More low-level alternative to [descendants()](#method.descendants).
    pub fn traverse(&self) -> Traverse {
        Traverse::new(self)
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    ///
    /// # Panics
    ///
    /// Panics if the node or one of its adjoining nodes is currently borrowed.
    pub fn detach(&mut self) {
        self.0.borrow_mut().detach();
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
    /// let doc = Document::from_str(
    /// "<svg>
    ///     <rect id='rect1'/>
    ///     <use xlink:href='#rect1'/>
    /// </svg>").unwrap();
    ///
    /// let mut rect_elem = doc.descendants().filter(|n| *n.id() == "rect1").next().unwrap();
    /// let use_elem = doc.descendants().filter(|n| n.is_tag_name(ElementId::Use)).next().unwrap();
    ///
    /// assert_eq!(use_elem.has_attribute(AttributeId::XlinkHref), true);
    ///
    /// // The 'remove' method will remove 'rect' element and all it's children.
    /// // Also it will remove all links to this element and it's children,
    /// // so 'use' element will no longer have the 'xlink:href' attribute.
    /// rect_elem.remove();
    ///
    /// assert_eq!(use_elem.has_attribute(AttributeId::XlinkHref), false);
    /// ```
    pub fn remove(&mut self) {
        let mut ids = Vec::with_capacity(16);
        Node::_remove(self, &mut ids);
    }

    fn _remove(node: &mut Node, ids: &mut Vec<AttributeId>) {
        ids.clear();

        for (aid, attr) in node.attributes().iter_svg() {
            match attr.value {
                AttributeValue::Link(_) | AttributeValue::FuncLink(_) => {
                    ids.push(aid)
                }
                _ => {}
            }
        }

        node.remove_attributes(&ids);

        // remove all attributes that linked to this node
        for mut linked in node.linked_nodes().collect::<Vec<Node>>() {
            ids.clear();

            for (aid, attr) in linked.attributes().iter_svg() {
                match attr.value {
                      AttributeValue::Link(ref link)
                    | AttributeValue::FuncLink(ref link) => {
                        if link == node {
                            ids.push(aid);
                        }
                    }
                    _ => {}
                }
            }

            linked.remove_attributes(&ids);
        }


        // repeat for children
        for mut child in node.children() {
            Node::_remove(&mut child, ids);
        }

        node.detach();
    }

    /// Removes only the children nodes specified by the predicate.
    ///
    /// Uses [remove()](#method.remove), not [detach()](#method.detach) internally.
    ///
    /// Current node ignored.
    pub fn drain<P>(&mut self, f: P) -> usize
        where P: Fn(&Node) -> bool
    {
        let mut count = 0;
        Node::_drain(self, &f, &mut count);
        count
    }

    fn _drain<P>(parent: &mut Node, f: &P, count: &mut usize)
        where P: Fn(&Node) -> bool
    {
        let mut node = parent.first_child();
        while let Some(mut n) = node {
            if f(&n) {
                node = n.next_sibling();
                n.remove();
                *count += 1;
            } else {
                if n.has_children() {
                    Node::_drain(&mut n, f, count);
                }

                node = n.next_sibling();
            }
        }
    }

    /// Returns a copy of a current node without children.
    ///
    /// All attributes except `id` will be copied, because `id` must be unique.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn make_copy(&self) -> Node {
        match self.node_type() {
            NodeType::Element => {
                let mut elem = self.document().create_element(self.tag_name().unwrap().into_ref());

                for attr in self.attributes().iter() {
                    elem.set_attribute(attr.clone());
                }

                elem
            }
            _ => {
                self.document().create_node(self.node_type(), &*self.text())
            }
        }
    }

    /// Returns a deep copy of a current node with all it's children.
    ///
    /// All attributes except `id` will be copied, because `id` must be unique.
    ///
    /// # Panics
    ///
    /// Panics if the node or any children node are currently mutably borrowed.
    pub fn make_deep_copy(&self) -> Node {
        let mut root = self.make_copy();
        Node::_make_deep_copy(&mut root, self);
        root
    }

    fn _make_deep_copy(parent: &mut Node, node: &Node) {
        for child in node.children() {
            let mut new_node = child.make_copy();
            parent.append(&new_node);

            if child.has_children() {
                Node::_make_deep_copy(&mut new_node, &child);
            }
        }
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new child, or one of their adjoining nodes is currently borrowed.
    pub fn append(&mut self, new_child: &Node) {
        let mut this = self.0.borrow_mut();
        let mut last = None;
        let nc = new_child.clone();
        {
            let mut child = nc.0.borrow_mut();
            child.detach();
            child.parent = Some(Rc::downgrade(&self.0));
            if let Some(last_weak) = this.last_child.take() {
                if let Some(last_strong) = last_weak.upgrade() {
                    child.prev_sibling = Some(last_weak);
                    last = Some(last_strong);
                }
            }
            this.last_child = Some(Rc::downgrade(&nc.0));
        }

        if let Some(last) = last {
            let mut last = last.borrow_mut();
            debug_assert!(last.next_sibling.is_none());
            last.next_sibling = Some(nc.0);
        } else {
            // No last child
            debug_assert!(this.first_child.is_none());
            this.first_child = Some(nc.0);
        }
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new child, or one of their adjoining nodes is currently borrowed.
    pub fn prepend(&mut self, new_child: &Node) {
        let mut this = self.0.borrow_mut();
        {
            let mut child = new_child.0.borrow_mut();
            child.detach();
            child.parent = Some(Rc::downgrade(&self.0));
            match this.first_child.take() {
                Some(first) => {
                    {
                        let mut first = first.borrow_mut();
                        debug_assert!(first.prev_sibling.is_none());
                        first.prev_sibling = Some(Rc::downgrade(&new_child.0));
                    }
                    child.next_sibling = Some(first);
                }
                None => {
                    debug_assert!(this.first_child.is_none());
                    this.last_child = Some(Rc::downgrade(&new_child.0));
                }
            }
        }
        this.first_child = Some(new_child.clone().0);
    }

    /// Insert a new sibling after this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_after(&mut self, new_sibling: &Node) {
        // TODO: add an example, since we need to detach 'new_sibling'
        //       before passing it to this method
        let mut this = self.0.borrow_mut();
        {
            let mut child = new_sibling.0.borrow_mut();
            child.detach();
            child.parent = this.parent.clone();
            child.prev_sibling = Some(Rc::downgrade(&self.0));
            match this.next_sibling.take() {
                Some(next) => {
                    {
                        let mut next = next.borrow_mut();
                        debug_assert!({
                            let weak = next.prev_sibling.as_ref().unwrap();
                            same_rc(&weak.upgrade().unwrap(), &self.0)
                        });
                        next.prev_sibling = Some(Rc::downgrade(&new_sibling.0));
                    }
                    child.next_sibling = Some(next);
                }
                None => {
                    Node::update_parent(&this, |mut p| {
                        p.last_child = Some(Rc::downgrade(&new_sibling.0))
                    });
                }
            }
        }
        this.next_sibling = Some(new_sibling.clone().0);
    }

    /// Insert a new sibling before this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_before(&mut self, new_sibling: &Node) {
        let mut this = self.0.borrow_mut();
        let mut prev_opt = None;
        {
            let mut child = new_sibling.0.borrow_mut();
            child.detach();
            child.parent = this.parent.clone();
            child.next_sibling = Some(Rc::clone(&self.0));
            if let Some(prev_weak) = this.prev_sibling.take() {
                if let Some(prev_strong) = prev_weak.upgrade() {
                    child.prev_sibling = Some(prev_weak);
                    prev_opt = Some(prev_strong);
                }
            }
            this.prev_sibling = Some(Rc::downgrade(&new_sibling.0));
        }

        if let Some(prev) = prev_opt {
            let mut prev = prev.borrow_mut();
            debug_assert!({
                let rc = prev.next_sibling.as_ref().unwrap();
                same_rc(rc, &self.0)
            });
            prev.next_sibling = Some(new_sibling.clone().0);
        } else {
            // No prev sibling.
            Node::update_parent(&this, |mut p| {
                p.first_child = Some(new_sibling.clone().0)
            });
        }
    }

    fn update_parent<F>(this: &RefMut<NodeData>, mut f: F)
        where F: FnMut(RefMut<NodeData>)
    {
        if let Some(parent) = this.parent.as_ref() {
            if let Some(parent) = parent.upgrade() {
                f(parent.borrow_mut());
            }
        }
    }

    /// Returns node's type.
    ///
    /// You can't change the type of the node. Only create a new one.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn node_type(&self) -> NodeType {
        self.0.borrow().node_type
    }

    /// Returns a text data of the node.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn text(&self) -> Ref<String> {
        Ref::map(self.0.borrow(), |n| &n.text)
    }

    /// Returns a mutable text data of the node.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn text_mut(&mut self) -> RefMut<String> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.text)
    }

    /// Sets a text data to the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn set_text(&mut self, text: &str) {
        debug_assert_ne!(self.node_type(), NodeType::Element);
        let mut b = self.0.borrow_mut();
        b.text = text.to_owned();
    }

    /// Returns an ID of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn id(&self) -> Ref<String> {
        Ref::map(self.0.borrow(), |n| &n.id)
    }

    /// Returns `true` if node has a not empty ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_id(&self) -> bool {
        !self.0.borrow().id.is_empty()
    }

    /// Sets an ID of the element.
    ///
    /// Only element nodes can contain an ID.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn set_id<S: Into<String>>(&mut self, id: S) {
        // TODO: check that it's unique.
        debug_assert_eq!(self.node_type(), NodeType::Element);
        self.0.borrow_mut().id = id.into();
    }

    /// Returns `true` if node has an `Element` type and an SVG tag name.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_svg_element(&self) -> bool {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref tag) => {
                match *tag {
                    Name::Id(_) => true,
                    Name::Name(_) => false,
                }
            }
            None => false,
        }
    }

    /// Returns a tag name of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn tag_name(&self) -> Option<Ref<TagName>> {
        // TODO: return NameRef somehow
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
    /// Panics if the node is currently mutably borrowed.
    pub fn tag_id(&self) -> Option<ElementId> {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref t) => {
                match *t {
                    Name::Id(ref id) => Some(*id),
                    Name::Name(_) => None,
                }
            }
            None => None,
        }
    }

    /// Returns `true` if node has the same tag name as supplied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_tag_name<'a, T>(&self, tag_name: T) -> bool
        where TagNameRef<'a>: From<T>
    {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref v) => v.into_ref() == TagNameRef::from(tag_name),
            None => false,
        }
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
    /// - Panics if the node is currently borrowed.
    /// - Panics if a string tag name is empty.
    pub fn set_tag_name<'a, T>(&mut self, tag_name: T)
        where TagNameRef<'a>: From<T>
    {
        debug_assert_eq!(self.node_type(), NodeType::Element);

        let tn = TagNameRef::from(tag_name);
        if let NameRef::Name(name) = tn {
            if name.is_empty() {
                panic!("supplied tag name is empty");
            }
        }

        let mut self_borrow = self.0.borrow_mut();
        self_borrow.tag_name = Some(Name::from(tn));
    }

    /// Returns a reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn attributes(&self) -> Ref<Attributes> {
        Ref::map(self.0.borrow(), |n| &n.attributes)
    }

    /// Returns a mutable reference to the `Attributes` of the current node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn attributes_mut(&mut self) -> RefMut<Attributes> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.attributes)
    }

    /// Returns `true` if the node has an attribute with such `id`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    #[inline]
    pub fn has_attribute<'a, N>(&self, name: N) -> bool
        where AttributeNameRef<'a>: From<N>
    {
        self.0.borrow().attributes.contains(name)
    }

    /// Returns `true` if the node has an attribute with such `id` and this attribute is visible.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_visible_attribute(&self, id: AttributeId) -> bool {
        if let Some(attr) = self.attributes().get(id) { attr.visible } else { false }
    }

    // TODO: remove
    /// Returns `true` if the node has any of provided attributes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn has_attributes(&self, ids: &[AttributeId]) -> bool {
        let attrs = self.attributes();
        for id in ids {
            if attrs.contains(*id) {
                return true;
            }
        }

        false
    }

    /// Inserts a new attribute into attributes list.
    ///
    /// Unwrapped version of the [`set_attribute_checked`] method.
    ///
    /// # Panics
    ///
    /// Will panic on any error produced by the [`set_attribute_checked`] method.
    ///
    /// [`set_attribute_checked`]: #method.set_attribute_checked
    pub fn set_attribute<T>(&mut self, v: T)
        where T: Into<Attribute>
    {
        self.set_attribute_checked(v).unwrap();
    }

    /// Inserts a new attribute into attributes list.
    ///
    /// You can set attribute using one of the possible combinations:
    ///
    /// - ([`AttributeId`]/`&str`, [`AttributeValue`])
    /// - ([`AttributeId`], [`Node`])
    /// - [`Attribute`]
    ///
    /// [`AttributeId`]: enum.AttributeId.html
    /// [`Attribute`]: struct.Attribute.html
    /// [`Node`]: struct.Node.html
    /// [`AttributeValue`]: enum.AttributeValue.html
    ///
    /// This method will overwrite an existing attribute with the same name.
    ///
    /// # Errors
    ///
    /// - [`ElementMustHaveAnId`]
    /// - [`ElementCrosslink`]
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    ///
    /// # Examples
    ///
    /// Ways to specify attributes:
    ///
    /// ```
    /// use svgdom::{
    ///     Document,
    ///     Attribute,
    ///     AttributeId as AId,
    ///     ElementId as EId,
    /// };
    ///
    /// // Create a simple document.
    /// let mut doc = Document::new();
    /// let mut svg = doc.create_element(EId::Svg);
    /// let mut rect = doc.create_element(EId::Rect);
    ///
    /// doc.append(&svg);
    /// svg.append(&rect);
    ///
    /// // In order to set element as an attribute value, we must set id first.
    /// rect.set_id("rect1");
    ///
    /// // Using predefined attribute name.
    /// svg.set_attribute((AId::X, 1.0));
    /// svg.set_attribute((AId::X, "random text"));
    /// // Using custom attribute name.
    /// svg.set_attribute(("non-svg-attr", 1.0));
    /// // Using existing attribute object.
    /// svg.set_attribute(Attribute::new(AId::X, 1.0));
    /// svg.set_attribute(Attribute::new("non-svg-attr", 1.0));
    /// // Using an existing node as an attribute value.
    /// svg.set_attribute((AId::XlinkHref, rect));
    /// ```
    ///
    /// Linked attributes:
    ///
    /// ```
    /// use svgdom::{
    ///     Document,
    ///     AttributeId as AId,
    ///     ElementId as EId,
    ///     ValueId,
    /// };
    ///
    /// // Create a simple document.
    /// let mut doc = Document::new();
    /// let mut gradient = doc.create_element(EId::LinearGradient);
    /// let mut rect = doc.create_element(EId::Rect);
    ///
    /// doc.append(&gradient);
    /// doc.append(&rect);
    ///
    /// gradient.set_id("lg1");
    /// rect.set_id("rect1");
    ///
    /// // Set a `fill` attribute value to the `none`.
    /// // For now everything like in any other XML DOM library.
    /// rect.set_attribute((AId::Fill, ValueId::None));
    ///
    /// // Now we want to fill our rect with a gradient.
    /// // To do this we need to set a link attribute:
    /// rect.set_attribute((AId::Fill, gradient.clone()));
    ///
    /// // Now our fill attribute has a link to the `gradient` node.
    /// // Not as text, aka `url(#lg1)`, but as actual reference.
    ///
    /// // This adds support for fast checking that the element is used. Which is very useful.
    ///
    /// // `gradient` is now used, since we link it.
    /// assert_eq!(gradient.is_used(), true);
    /// // Also, we can check how many elements are uses this `gradient`.
    /// assert_eq!(gradient.uses_count(), 1);
    /// // And even get this elements.
    /// assert_eq!(gradient.linked_nodes().next().unwrap(), rect);
    ///
    /// // And now, if we remove our `rect` - `gradient` will became unused again.
    /// rect.remove();
    /// assert_eq!(gradient.is_used(), false);
    /// ```
    ///
    /// [`ElementMustHaveAnId`]: enum.Error.html
    /// [`ElementCrosslink`]: enum.Error.html
    pub fn set_attribute_checked<T>(&mut self, v: T) -> Result<()>
        where T: Into<Attribute>
    {
        self.set_attribute_checked_impl(v.into())
    }

    fn set_attribute_checked_impl(&mut self, attr: Attribute) -> Result<()> {
        // TODO: to error in _checked mode
        debug_assert_eq!(self.node_type(), NodeType::Element);

        if attr.is_svg() {
            match attr.value {
                  AttributeValue::Link(ref iri)
                | AttributeValue::FuncLink(ref iri) => {
                    let aid = attr.id().unwrap();
                    self.set_link_attribute(aid, iri.clone())?;
                    return Ok(());
                }
                _ => {}
            }
        }

        self.set_simple_attribute(attr);

        Ok(())
    }

    fn set_simple_attribute(&mut self, attr: Attribute) {
        debug_assert!(!attr.is_link() && !attr.is_func_link());

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(attr.name.into_ref());

        let mut attrs = self.attributes_mut();
        attrs.insert(attr);
    }

    fn set_link_attribute(&mut self, id: AttributeId, node: Node) -> Result<()> {
        if node.id().is_empty() {
            return Err(ErrorKind::ElementMustHaveAnId.into());
        }

        // check for recursion
        if *self.id() == *node.id() {
            return Err(ErrorKind::ElementCrosslink.into());
        }

        // check for recursion 2
        if self.linked_nodes().any(|n| n == node) {
            return Err(ErrorKind::ElementCrosslink.into());
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
            attributes.insert_impl(a);
        }

        {
            let mut value_borrow = node.0.borrow_mut();
            value_borrow.linked_nodes.push(Rc::downgrade(&self.0));
        }

        Ok(())
    }

    /// Inserts a new attribute into attributes list if it doesn't contain one.
    ///
    /// `value` will be cloned if needed.
    ///
    /// Shorthand for:
    ///
    /// ```ignore
    /// if !node.has_attribute(...) {
    ///     node.set_attribute(...);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Will panic on any error produced by the [`set_attribute_checked`] method.
    ///
    /// [`set_attribute_checked`]: #method.set_attribute_checked
    pub fn set_attribute_if_none<'a, N, T>(&mut self, name: N, value: &T)
        where AttributeNameRef<'a>: From<N>, N: Copy, AttributeValue: From<T>, T: Clone
    {
        if !self.has_attribute(name) {
            self.set_attribute((name, value.clone()));
        }
    }

    /// Removes an attribute from the node.
    ///
    /// It will also unlink it, if it was an referenced attribute.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attribute<'a, N>(&mut self, name: N)
        where AttributeNameRef<'a>: From<N>, N: Copy
    {
        if !self.has_attribute(name) {
            return;
        }

        // we must unlink referenced attributes
        if let Some(value) = self.attributes().get_value(name) {
            match *value {
                AttributeValue::Link(ref node) | AttributeValue::FuncLink(ref node) => {
                    let self_borrow = &self.0;
                    let mut node_borrow = node.0.borrow_mut();
                    let ln = &mut node_borrow.linked_nodes;
                    // this code can't panic, because we know that such node exist
                    let index = ln.iter().position(|x| {
                        same_rc(&x.upgrade().unwrap(), self_borrow)
                    }).unwrap();
                    ln.remove(index);
                }
                _ => {}
            }
        }

        self.attributes_mut().remove_impl(name);
    }

    // TODO: remove
    /// Removes attributes from the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attributes(&mut self, ids: &[AttributeId]) {
        // TODO: to AttributeNameRef
        for id in ids {
            self.remove_attribute(*id);
        }
    }

    /// Returns an iterator over linked nodes.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn linked_nodes(&self) -> LinkedNodes {
        LinkedNodes::new(Ref::map(self.0.borrow(), |n| &n.linked_nodes))
    }

    /// Returns `true` if the current node is linked to any of the DOM nodes.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn is_used(&self) -> bool {
        let self_borrow = self.0.borrow();
        !self_borrow.linked_nodes.is_empty()
    }

    /// Returns a number of nodes, which is linked to this node.
    ///
    /// See [Node::set_attribute()](#method.set_attribute) for details.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutably borrowed.
    pub fn uses_count(&self) -> usize {
        let self_borrow = self.0.borrow();
        self_borrow.linked_nodes.len()
    }
}

/// Cloning a `Node` only increments a reference count. It does not copy the data.
impl Clone for Node {
    fn clone(&self) -> Node {
        Node(Rc::clone(&self.0))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        same_rc(&self.0, &other.0)
    }
}

// TODO: move to Rc::ptr_eq (since 1.17) when we drop 1.13 version support
fn same_rc<T>(a: &Rc<T>, b: &Rc<T>) -> bool {
    let a: *const T = &**a;
    let b: *const T = &**b;
    a == b
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type() {
            NodeType::Root => write!(f, "RootNode"),
            NodeType::Element => write!(f, "ElementNode({:?} id={:?})", self.tag_name().unwrap(), self.id()),
            NodeType::Declaration => write!(f, "DeclarationNode({:?})", *self.text()),
            NodeType::Comment => write!(f, "CommentNode({:?})", *self.text()),
            NodeType::Cdata => write!(f, "CdataNode({:?})", *self.text()),
            NodeType::Text => write!(f, "TextNode({:?})", *self.text()),
        }
    }
}
