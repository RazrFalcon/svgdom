// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/*!
`rcc-tree` is a "DOM-like" tree implemented using custom reference counting that allows cycles.

"DOM-like" here means that data structures can be used to represent
the parsed content of an HTML or XML document,
like [*the* DOM](https://dom.spec.whatwg.org/) does,
but don't necessarily have the exact same API as the DOM.
That is:

* A tree is made up of nodes.
* Each node has zero or more *child* nodes, which are ordered.
* Each node has a no more than one *parent*, the node that it is a *child* of.
* A node without a *parent* is called a *root*.
* As a consequence, each node may also have *siblings*: its *parent*'s other *children*, if any.
* From any given node, access to its
  parent, previous sibling, next sibling, first child, and last child (if any)
  can take no more than *O(1)* time.
* Each node also has data associated to it,
  which for the purpose of this project is purely generic.
  For an HTML document, the data would be either the text of a text node,
  or the name and attributes of an element node.
* The tree is mutable:
  nodes (with their sub-trees) can be inserted or removed anywhere in the tree.

The lifetime of nodes is managed through *reference counting*.
To avoid reference cycles which would cause memory leaks, the tree is using
a custom reference counting implementation.

* Nodes can be created only through the `Document` object to which all of them belong.
* Nodes can reference other nodes. Cycles are allowed.
* Accessing a `Node` after the owning `Document` goes out of scope will lead to a panic.
* Accessing a `Node` after it was removed will lead to a panic.
* A `Node` lives as long as a `Document` which created it.
* Adjoining nodes can be borrowed mutably at the same time (will lead to a panic in `rctree`).
* Nodes can be borrowed mutably indirectly multiple times.
  It's a bit against the Rust philosophy, but it makes life easier.
  You can use `RefCell` to prevent this.

### Safety

* The library may panic on a contract violation.
* The library uses unsafe code, so use-after-free, double-free and memory leaks are possible.
  Despite that, all tests are passed under the `RUSTFLAGS="-Z sanitizer=address"`.

*/

// Changes:
// - `Node::borrow` and `Node::borrow_mut` marked as `pub(crate)`.

extern crate slab;

use std::fmt;

use self::slab::Slab;

/// Iterators prelude.
pub mod iterators {
    pub use super::Ancestors;
    pub use super::PrecedingSiblings;
    pub use super::FollowingSiblings;
    pub use super::Children;
    pub use super::ReverseChildren;
    pub use super::Descendants;
    pub use super::Traverse;
    pub use super::ReverseTraverse;
    pub use super::NodeEdge;
}

struct NodeData<T> {
    /// References count.
    count: usize,
    /// Key to a document storage.
    ///
    /// If set to `None` then it was removed and became invalid.
    key: Option<usize>,

    root: Option<*mut NodeData<T>>,
    parent: Option<*mut NodeData<T>>,
    first_child: Option<*mut NodeData<T>>,
    last_child: Option<*mut NodeData<T>>,
    previous_sibling: Option<*mut NodeData<T>>,
    next_sibling: Option<*mut NodeData<T>>,

    /// An actual node's data.
    data: T,
}

impl<T> NodeData<T> {
    /// Returns `true` if a key to a document storage is set.
    fn is_valid(&self) -> bool {
        self.key.is_some()
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    fn detach(&mut self) {
        let parent = self.parent.take();
        let previous_sibling = self.previous_sibling.take();
        let next_sibling = self.next_sibling.take();
        let previous_sibling_opt = previous_sibling;

        unsafe {
            if let Some(next_sibling) = next_sibling {
                (*next_sibling).previous_sibling = previous_sibling;
            } else if let Some(parent) = parent {
                (*parent).last_child = previous_sibling;
            }

            if let Some(previous_sibling) = previous_sibling_opt {
                (*previous_sibling).next_sibling = next_sibling;
            } else if let Some(parent) = parent {
                (*parent).first_child = next_sibling;
            }
        }
    }

    fn reset(&mut self) {
        self.key = None;
        self.parent = None;
        self.first_child = None;
        self.last_child = None;
        self.previous_sibling = None;
        self.next_sibling = None;
    }
}

/// A node holding a value of type `T`.
///
/// Lives as long as the `Document` that created it.
/// As soon as the `Document` goes out of scope - all its nodes became "invalid".
/// This means that they don't have parent, children, siblings and they will produce a panic
/// on access.
///
/// # Panics
///
/// - All methods will panic while accessing the node after the document or the node itself removal.
pub struct Node<T>(*mut NodeData<T>);

/// Cloning a `Node` only increments a reference count. It does not copy the data.
impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node::new(self.0)
    }
}

/// Compares pointers, not data.
impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.0 == other.0
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        unsafe {
            if (*self.0).count == 1 {
                Box::from_raw(self.0);
            } else {
                (*self.0).count -= 1;
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&*self.borrow(), f)
    }
}

impl<T> Node<T> {
    fn new(data: *mut NodeData<T>) -> Self {
        unsafe { (*data).count += 1; }
        Node(data)
    }

    fn get(&self) -> &NodeData<T> {
        unsafe {
            assert!((*self.0).is_valid(), "node was already removed");
            &(*self.0)
        }
    }

    fn get_mut(&mut self) -> &mut NodeData<T> {
        unsafe {
            assert!((*self.0).is_valid(), "node was already removed");
            &mut (*self.0)
        }
    }

    /// Returns a current node data.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub(crate) fn borrow(&self) -> &T {
        &self.get().data
    }

    /// Returns a current node mutable data.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub(crate) fn borrow_mut(&mut self) -> &mut T {
        &mut self.get_mut().data
    }

    /// Returns a root node.
    ///
    /// If the current node is the root node - will return itself.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn root(&self) -> Node<T> {
        match self.get().root {
            Some(v) => Node::new(v),
            None => self.clone(),
        }
    }

    /// Returns a parent node, unless this node is the root of the tree.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn parent(&self) -> Option<Node<T>> {
        self.get().parent.map(Node::new)
    }

    /// Returns a first child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn first_child(&self) -> Option<Node<T>> {
        self.get().first_child.map(Node::new)
    }

    /// Returns a last child of this node, unless it has no child.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn last_child(&self) -> Option<Node<T>> {
        self.get().last_child.map(Node::new)
    }

    /// Returns a previous sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn previous_sibling(&self) -> Option<Node<T>> {
        self.get().previous_sibling.map(Node::new)
    }

    /// Returns a previous sibling of this node, unless it is a first child.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn next_sibling(&self) -> Option<Node<T>> {
        self.get().next_sibling.map(Node::new)
    }

    /// Returns an iterator of nodes to this node and its ancestors.
    ///
    /// Includes the current node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn ancestors(&self) -> Ancestors<T> {
        Ancestors(Some(self.clone()))
    }

    /// Returns an iterator of nodes to this node and the siblings before it.
    ///
    /// Includes the current node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn preceding_siblings(&self) -> PrecedingSiblings<T> {
        PrecedingSiblings(Some(self.clone()))
    }

    /// Returns an iterator of nodes to this node and the siblings after it.
    ///
    /// Includes the current node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn following_siblings(&self) -> FollowingSiblings<T> {
        FollowingSiblings(Some(self.clone()))
    }

    /// Returns an iterator of nodes to this node's children.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn children(&self) -> Children<T> {
        Children(self.first_child())
    }

    /// Returns `true` if a node has children nodes.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn has_children(&self) -> bool {
        self.first_child().is_some()
    }

    /// Returns an iterator of nodes to this node's children, in reverse order.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn reverse_children(&self) -> ReverseChildren<T> {
        ReverseChildren(self.last_child())
    }

    /// Returns an iterator of nodes to this node and its descendants, in tree order.
    ///
    /// Includes the current node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn descendants(&self) -> Descendants<T> {
        Descendants(self.traverse())
    }

    /// Returns an iterator of nodes to this node and its descendants, in tree order.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn traverse(&self) -> Traverse<T> {
        Traverse {
            root: self.clone(),
            next: Some(NodeEdge::Start(self.clone())),
        }
    }

    /// Returns an iterator of nodes to this node and its descendants, in tree order.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn reverse_traverse(&self) -> ReverseTraverse<T> {
        ReverseTraverse {
            root: self.clone(),
            next: Some(NodeEdge::End(self.clone())),
        }
    }

    /// Appends a new child to this node, after existing children.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    /// - If the node and a `new_child` is the same node.
    /// - If the node and a `new_child` belong to different documents.
    pub fn append(&mut self, mut new_child: Node<T>) {
        assert!(*self != new_child, "a node cannot be appended to itself");
        assert!(self.root() == new_child.root(),
                "a node cannot be appended to a different document");

        let mut last_child_opt = None;
        new_child.detach();
        new_child.get_mut().parent = Some(self.0);
        if let Some(last_child) = self.get_mut().last_child.take() {
            new_child.get_mut().previous_sibling = Some(last_child);
            last_child_opt = Some(Node::new(last_child));
        }
        self.get_mut().last_child = Some(new_child.0);

        if let Some(mut last_child) = last_child_opt {
            debug_assert!(last_child.get().next_sibling.is_none());
            last_child.get_mut().next_sibling = Some(new_child.0);
        } else {
            // No last child
            debug_assert!(self.get().first_child.is_none());
            self.get_mut().first_child = Some(new_child.0);
        }
    }

    /// Prepends a new child to this node, before existing children.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    /// - If the node and a `new_child` is the same node.
    /// - If the node and a `new_child` belong to different documents.
    pub fn prepend(&mut self, mut new_child: Node<T>) {
        assert!(*self != new_child, "a node cannot be prepended to itself");
        assert!(self.root() == new_child.root(),
                "a node cannot be prepended to a different document");

        new_child.detach();
        new_child.get_mut().parent = Some(self.0);
        match self.get_mut().first_child.take() {
            Some(first_child) => {
                let mut first_child_node = Node::new(first_child);
                debug_assert!(first_child_node.get().previous_sibling.is_none());
                first_child_node.get_mut().previous_sibling = Some(new_child.0);
                new_child.get_mut().next_sibling = Some(first_child);
            }
            None => {
                debug_assert!(self.get().first_child.is_none());
                self.get_mut().last_child = Some(new_child.0);
            }
        }
        self.get_mut().first_child = Some(new_child.0);
    }


    /// Inserts a new sibling after this node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    /// - If the node and a `new_sibling` is the same node.
    /// - If the node and a `new_sibling` belong to different documents.
    pub fn insert_after(&mut self, mut new_sibling: Node<T>) {
        assert!(*self != new_sibling, "a node cannot be inserted after itself");
        assert!(self.root() == new_sibling.root(),
                "a node cannot be inserted to a different document");

        new_sibling.detach();
        new_sibling.get_mut().parent = self.get().parent.clone();
        new_sibling.get_mut().previous_sibling = Some(self.0);
        match self.get_mut().next_sibling.take() {
            Some(next_sibling) => {
                let mut next_sibling_node = Node::new(next_sibling);
                next_sibling_node.get_mut().previous_sibling = Some(new_sibling.0);
                new_sibling.get_mut().next_sibling = Some(next_sibling);
            }
            None => {
                if let Some(parent) = self.get_mut().parent {
                    Node::new(parent).get_mut().last_child = Some(new_sibling.0);
                }
            }
        }
        self.get_mut().next_sibling = Some(new_sibling.0);
    }

    /// Inserts a new sibling before this node.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    /// - If the node and a `new_sibling` is the same node.
    /// - If the node and a `new_sibling` belong to different documents.
    pub fn insert_before(&mut self, mut new_sibling: Node<T>) {
        assert!(*self != new_sibling, "a node cannot be inserted before itself");
        assert!(self.root() == new_sibling.root(),
                "a node cannot be inserted to a different document");

        let mut previous_sibling_opt = None;
        {
            new_sibling.detach();
            new_sibling.get_mut().parent = self.get().parent;
            new_sibling.get_mut().next_sibling = Some(self.0);
            if let Some(previous_sibling) = self.get_mut().previous_sibling.take() {
                new_sibling.get_mut().previous_sibling = Some(previous_sibling);
                previous_sibling_opt = Some(Node::new(previous_sibling));
            }
            self.get_mut().previous_sibling = Some(new_sibling.0);
        }

        if let Some(mut previous_sibling) = previous_sibling_opt {
            previous_sibling.get_mut().next_sibling = Some(new_sibling.0);
        } else {
            // No previous sibling.
            if let Some(parent) = self.get_mut().parent {
                Node::new(parent).get_mut().first_child = Some(new_sibling.0);
            }
        }
    }

    /// Detaches a node from its parent and siblings. Children are not affected.
    ///
    /// # Panics
    ///
    /// - If the node was removed.
    pub fn detach(&mut self) {
        self.get_mut().detach();
    }
}

/// A nodes container.
pub struct Document<T> {
    root: *mut NodeData<T>,
    storage: Slab<Node<T>>,
}

impl<T> Drop for Document<T> {
    fn drop(&mut self) {
        for (_, node) in &mut self.storage {
            node.get_mut().reset();
        }
    }
}

impl<T> Document<T> {
    /// Creates a new `Document` using provided root node data.
    pub fn new(data: T) -> Self {
        let mut storage = Slab::new();
        let root_data = Box::new(NodeData {
            count: 0,
            root: None,
            key: None,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data,
        });

        let root_data_raw: *mut _ = Box::into_raw(root_data);
        let key = storage.insert(Node::new(root_data_raw));
        unsafe { (*root_data_raw).key = Some(key);}

        Document {
            root: root_data_raw,
            storage,
        }
    }

    /// Creates a new node.
    ///
    /// Node belongs to the tree, but not added to it.
    ///
    /// See: [Node::append](struct.Node.html#method.append),
    /// [Node::prepend](struct.Node.html#method.prepend),
    /// [Node::insert_after](struct.Node.html#method.insert_after) and
    /// [Node::insert_before](struct.Node.html#method.insert_before).
    pub fn create_node(&mut self, data: T) -> Node<T> {
        let new_data = Box::new(NodeData {
            count: 0,
            root: Some(self.root),
            key: None,
            parent: None,
            first_child: None,
            last_child: None,
            previous_sibling: None,
            next_sibling: None,
            data,
        });

        let new_data_raw: *mut _ = Box::into_raw(new_data);
        let key = self.storage.insert(Node::new(new_data_raw));
        unsafe { (*new_data_raw).key = Some(key);}

        Node::new(new_data_raw)
    }

    /// Removes a provided node.
    ///
    /// The node will be detached from the document/tree and marked as removed.
    /// If you try to access a copy of it - you will get a panic.
    ///
    /// # Panics
    ///
    /// - If the root node is about to be removed.
    /// - If the node was already removed.
    /// - If the node belongs to a different document.
    pub fn remove_node(&mut self, mut node: Node<T>) {
        assert!(self.root != node.0, "the root node cannot be removed");
        assert!(self.root == node.root().0,
                "a node cannot be removed from a different document");

        node.detach();
        self.storage.remove(node.get_mut().key.take().unwrap());
    }

    /// Returns the root node.
    pub fn root(&self) -> Node<T> {
        Node::new(self.root)
    }
}

macro_rules! impl_node_iterator {
    ($name: ident, $next: expr) => {
        impl<T> Iterator for $name<T> {
            type Item = Node<T>;

            fn next(&mut self) -> Option<Self::Item> {
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

/// An iterator of nodes to the ancestors a given node.
pub struct Ancestors<T>(Option<Node<T>>);
impl_node_iterator!(Ancestors, |node: &Node<T>| node.parent());

/// An iterator of nodes to the siblings before a given node.
pub struct PrecedingSiblings<T>(Option<Node<T>>);
impl_node_iterator!(PrecedingSiblings, |node: &Node<T>| node.previous_sibling());

/// An iterator of nodes to the siblings after a given node.
pub struct FollowingSiblings<T>(Option<Node<T>>);
impl_node_iterator!(FollowingSiblings, |node: &Node<T>| node.next_sibling());

/// An iterator of nodes to the children of a given node.
pub struct Children<T>(Option<Node<T>>);
impl_node_iterator!(Children, |node: &Node<T>| node.next_sibling());

/// An iterator of nodes to the children of a given node, in reverse order.
pub struct ReverseChildren<T>(Option<Node<T>>);
impl_node_iterator!(ReverseChildren, |node: &Node<T>| node.previous_sibling());


/// An iterator of nodes to a given node and its descendants, in tree order.
pub struct Descendants<T>(Traverse<T>);

impl<T> Iterator for Descendants<T> {
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
}


/// A node type during traverse.
#[derive(Clone)]
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node's descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(Node<T>),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node's descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(Node<T>),
}


/// An iterator of nodes to a given node and its descendants, in tree order.
pub struct Traverse<T> {
    root: Node<T>,
    next: Option<NodeEdge<T>>,
}

impl<T> Iterator for Traverse<T> {
    type Item = NodeEdge<T>;

    fn next(&mut self) -> Option<Self::Item> {
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
                        if *node == self.root {
                            None
                        } else {
                            match node.next_sibling() {
                                Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                                None => match node.parent() {
                                    Some(parent) => Some(NodeEdge::End(parent)),

                                    // `node.parent()` here can only be `None`
                                    // if the tree has been modified during iteration,
                                    // but silently stoping iteration
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

/// An iterator of nodes to a given node and its descendants, in reverse tree order.
pub struct ReverseTraverse<T> {
    root: Node<T>,
    next: Option<NodeEdge<T>>,
}

impl<T> Iterator for ReverseTraverse<T> {
    type Item = NodeEdge<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(item) => {
                self.next = match item {
                    NodeEdge::End(ref node) => {
                        match node.last_child() {
                            Some(last_child) => Some(NodeEdge::End(last_child)),
                            None => Some(NodeEdge::Start(node.clone()))
                        }
                    }
                    NodeEdge::Start(ref node) => {
                        if *node == self.root {
                            None
                        } else {
                            match node.previous_sibling() {
                                Some(previous_sibling) => Some(NodeEdge::End(previous_sibling)),
                                None => match node.parent() {
                                    Some(parent) => Some(NodeEdge::Start(parent)),

                                    // `node.parent()` here can only be `None`
                                    // if the tree has been modified during iteration,
                                    // but silently stoping iteration
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
