// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::node::Node;
use super::node_data::WeakLink;
use super::node_type::NodeType;

pub struct LinkAttributes {
    data: Vec<WeakLink>,
    idx: usize,
}

impl LinkAttributes {
    pub fn new(data: Vec<WeakLink>) -> LinkAttributes {
        LinkAttributes {
            data: data,
            idx: 0,
        }
    }
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

impl Traverse {
    /// Constructs a new Traverse iterator.
    pub fn new(node: &Node) -> Traverse {
        Traverse {
            root: node.clone(),
            next: Some(NodeEdge::Start(node.clone())),
        }
    }
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

impl DescendantNodes {
    pub fn new(node: &Node) -> DescendantNodes {
        DescendantNodes(node.traverse())
    }
}

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
    pub fn new(node: &Node) -> Descendants {
        Descendants(node.traverse())
    }
}

impl Descendants {
    // TODO: find a better way
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
        impl $name {
            pub fn new(node: Option<Node>) -> $name {
                $name(node)
            }
        }

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
