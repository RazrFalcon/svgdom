// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::iter::Filter;

use super::node::Node;
use super::node_data::WeakLink;

/// Node type during traverse.
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
pub struct Descendants(Traverse);

impl Descendants {
    /// Constructs a new Descendants iterator.
    pub fn new(node: &Node) -> Descendants {
        Descendants(node.traverse())
    }
}

impl Descendants {
    /// Returns an iterator over descendant SVG elements.
    ///
    /// Shorthand for: `filter(|n| n.is_svg_element())`
    pub fn svg(self) -> Filter<Descendants, fn(&Node) -> bool> {
        fn is_svg(n: &Node) -> bool { n.is_svg_element() }
        self.filter(is_svg)
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
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None
            }
        }
    }
}

/// An iterator of `Node`s to the children of a given node.
pub struct Children(Option<Node>);

impl Children {
    /// Constructs a new Children iterator.
    pub fn new(node: Option<Node>) -> Children {
        Children(node)
    }
}

impl Iterator for Children {
    type Item = Node;

    /// # Panics
    ///
    /// Panics if the node about to be yielded is currently mutability borrowed.
    fn next(&mut self) -> Option<Node> {
        match self.0.take() {
            Some(node) => {
                self.0 = node.next_sibling();
                Some(node)
            }
            None => None
        }
    }
}

impl Children {
    /// Returns an iterator over children SVG elements.
    ///
    /// Shorthand for: `filter(|n| n.is_svg_element())`
    pub fn svg(self) -> Filter<Children, fn(&Node) -> bool> {
        fn is_svg(n: &Node) -> bool { n.is_svg_element() }
        self.filter(is_svg)
    }
}

/// An iterator over linked nodes.
pub struct LinkedNodes {
    data: Vec<WeakLink>,
    idx: usize,
}

impl LinkedNodes {
    /// Constructs a new LinkedNodes iterator.
    pub fn new(data: Vec<WeakLink>) -> LinkedNodes {
        LinkedNodes {
            data: data,
            idx: 0,
        }
    }
}

impl Iterator for LinkedNodes {
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
