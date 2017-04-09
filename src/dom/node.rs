// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use std::fmt;

use attribute::*;
use {
    AttributeId,
    ElementId,
    Error,
    Name,
    NameRef,
    SvgId,
    TagName,
    TagNameRef,
};
use super::document::Document;
use super::iterators::*;
use super::node_data::NodeData;
use super::node_type::NodeType;

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
// TODO: maybe copyable
pub struct Node(pub Rc<RefCell<NodeData>>);

impl Node {
    /// Returns a `Document` that owns this node.
    ///
    /// # Panics
    ///
    /// - Panics if the node is currently mutability borrowed.
    /// - Panics if the node is a root node.
    pub fn document(&self) -> Document {
        // TODO: will fail on root node
        Document { root: Node(self.0.borrow().doc.as_ref().unwrap().upgrade().unwrap()) }
    }

    /// Returns a parent node, unless this node is the root of the tree.
    ///
    /// This method also returns `NodeType::Root`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn parent(&self) -> Option<Node> {
        Some(Node(try_opt!(try_opt!(self.0.borrow().parent.as_ref()).upgrade())))
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

    /// Returns an iterator over node's parents.
    ///
    /// Current node is not included.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn parents(&self) -> Parents {
        Parents::new(self.parent())
    }

    /// Returns an iterator to this node's children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn children(&self) -> Children {
        Children::new(self.first_child())
    }

    /// Returns `true` if this node has children nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn has_children(&self) -> bool {
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

    /// Returns an iterator over descendant nodes.
    pub fn descendants(&self) -> Descendants {
        Descendants::new(self)
    }

    /// Returns an iterator over descendant nodes.
    ///
    /// More complex alternative of the `Node::descendants()`.
    pub fn traverse(&self) -> Traverse {
        Traverse::new(self)
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
        let mut ids: Vec<AttributeId> = node.attributes().iter_svg()
                                        .filter(|&(_, a)| a.is_link() || a.is_func_link())
                                        .map(|(id, _)| id)
                                        .collect();
        for id in &ids {
            node.remove_attribute(*id);
        }

        // remove all attributes that linked to this node
        for linked in node.linked_nodes().collect::<Vec<Node>>() {
            ids.clear();

            for (aid, attr) in linked.attributes().iter_svg() {
                match attr.value {
                    AttributeValue::Link(ref link) | AttributeValue::FuncLink(ref link) => {
                        if link == node {
                            ids.push(aid);
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
        for child in node.children().svg() {
            Node::_remove(&child);
        }
    }

    /// Removes only the children nodes specified by the predicate.
    ///
    /// Current node ignored.
    pub fn drain<P>(&self, f: P) -> usize
        where P: Fn(&Node) -> bool
    {
        let mut count = 0;
        Node::_drain(self, &f, &mut count);
        count
    }

    fn _drain<P>(parent: &Node, f: &P, count: &mut usize)
        where P: Fn(&Node) -> bool
    {
        let mut node = parent.first_child();
        while let Some(n) = node {
            if f(&n) {
                node = n.next_sibling();
                n.remove();
                *count += 1;
            } else {
                if n.has_children() {
                    Node::_drain(&n, f, count);
                }

                node = n.next_sibling();
            }
        }
    }

    /// Returns a copy of a current node without children.
    ///
    /// All attributes except 'id' will be copied.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn make_copy(&self) -> Node {
        match self.node_type() {
            NodeType::Element => {
                let elem = self.document().create_element(self.tag_name().unwrap().into_ref());

                for attr in self.attributes().iter() {
                    elem.set_attribute_object(attr.clone());
                }

                elem
            }
            _ => {
                self.document().create_node(self.node_type(), &*self.text().unwrap())
            }
        }
    }

    /// Returns a deep copy of a current node without children.
    ///
    /// All attributes except 'id' will be copied.
    ///
    /// # Panics
    ///
    /// Panics if the node or any children node are currently mutability borrowed.
    pub fn make_deep_copy(&self) -> Node {
        let root = self.make_copy();
        Node::_make_deep_copy(&root, self);
        return root;
    }

    fn _make_deep_copy(parent: &Node, node: &Node) {
        for child in node.children() {
            let new_node = child.make_copy();
            parent.append(&new_node);

            if child.has_children() {
                Node::_make_deep_copy(&new_node, &child);
            }
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
    pub fn prepend(&self, new_child: &Node) {
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
        self_borrow.first_child = Some(new_child.clone().0);
    }

    /// Insert a new sibling after this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_after(&self, new_sibling: &Node) {
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
        self_borrow.next_sibling = Some(new_sibling.clone().0);
    }

    /// Insert a new sibling before this node.
    ///
    /// # Panics
    ///
    /// Panics if the node, the new sibling, or one of their adjoining nodes is currently borrowed.
    pub fn insert_before(&self, new_sibling: &Node) {
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
            previous_sibling_borrow.next_sibling = Some(new_sibling.clone().0);
        } else {
            // No previous sibling.
            if let Some(parent_ref) = self_borrow.parent.as_ref() {
                if let Some(parent_strong) = parent_ref.upgrade() {
                    let mut parent_borrow = parent_strong.borrow_mut();
                    parent_borrow.first_child = Some(new_sibling.clone().0);
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

    /// Returns a text data of the node, if there are any.
    ///
    /// Nodes with `Element` type can't contain text data.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn text(&self) -> Option<Ref<String>> {
        match self.0.borrow().text {
            Some(_) => Some(Ref::map(self.0.borrow(), |n| n.text.as_ref().unwrap())),
            None => None,
        }
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
                    Name::Id(_) => true,
                    Name::Name(_) => false,
                }
            }
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
    pub fn set_tag_name<'a, T>(&self, tag_name: T)
        where TagNameRef<'a>: From<T>
    {
        debug_assert!(self.node_type() == NodeType::Element);

        let tn = TagNameRef::from(tag_name);
        if let NameRef::Name(ref name) = tn {
            if name.is_empty() {
                panic!("supplied tag name is empty");
            }
        }

        let mut self_borrow = self.0.borrow_mut();
        self_borrow.tag_name = Some(Name::from(tn));
    }

    /// Returns a tag name of the element node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
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
    /// Panics if the node is currently mutability borrowed.
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
    /// Panics if the node is currently mutability borrowed.
    pub fn is_tag_name<'a, T>(&self, tag_name: T) -> bool
        where TagNameRef<'a>: From<T>
    {
        let b = self.0.borrow();
        match b.tag_name {
            Some(ref v) => v.into_ref() == TagNameRef::from(tag_name),
            None => false,
        }
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
    pub fn set_attribute<'a, N, T>(&self, name: N, value: T)
        where AttributeNameRef<'a>: From<N>, N: Copy, AttributeValue: From<T>
    {
        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(name);

        let a = Attribute::new(name, value);
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
    /// - Panics if the attribute cause an ElementCrosslink error.
    pub fn set_attribute_object(&self, attr: Attribute) {
        // TODO: fix stupid name
        // TODO: do not panic on invalid attribute type

        // we must remove existing attribute to prevent dangling links
        self.remove_attribute(attr.name.into_ref());

        if attr.is_svg() {
            match attr.value {
                AttributeValue::Link(ref iri) | AttributeValue::FuncLink(ref iri) => {
                    let aid = attr.id().unwrap();
                    self.set_link_attribute(aid, iri.clone()).unwrap();
                    return;
                }
                _ => {}
            }
        }

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

    /// Returns `true` if the node has an attribute with such `id`.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
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
    pub fn remove_attribute<'a, N>(&self, name: N)
        where AttributeNameRef<'a>: From<N>, N: Copy
    {
        if !self.has_attribute(name) {
            return;
        }

        let mut attrs = self.attributes_mut();

        // we must unlink referenced attributes
        if let Some(value) = attrs.get_value(name) {
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

        attrs.remove(name);
    }

    /// Removes attributes from the node.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently borrowed.
    pub fn remove_attributes(&self, ids: &[AttributeId]) {
        // TODO: to AttributeNameRef
        for id in ids {
            self.remove_attribute(*id);
        }
    }

    /// Returns an iterator over linked nodes.
    ///
    /// # Panics
    ///
    /// Panics if the node is currently mutability borrowed.
    pub fn linked_nodes(&self) -> LinkedNodes {
        LinkedNodes::new(Ref::map(self.0.borrow(), |n| &n.linked_nodes))
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
        same_rc(&self.0, &other.0)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type() {
            NodeType::Root => write!(f, "Root node"),
            NodeType::Element => write!(f, "<{:?} id={:?}>", self.tag_name().unwrap(), self.id()),
            NodeType::Declaration => write!(f, "<?{}?>", *self.text().unwrap()),
            NodeType::Comment => write!(f, "<!--{}-->", *self.text().unwrap()),
            NodeType::Cdata => write!(f, "<![CDATA[{}]]>", *self.text().unwrap()),
            NodeType::Text => write!(f, "{}", *self.text().unwrap()),
        }
    }
}
