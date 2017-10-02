// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::rc::{
    Rc,
    Weak,
};

use {
    Attributes,
    TagName,
    NodeType,
    Node,
};

pub type Link = Rc<RefCell<NodeData>>;
pub type WeakLink = Weak<RefCell<NodeData>>;

pub struct NodeData {
    // TODO: check that doc is equal in append, insert, etc.
    pub doc: Option<WeakLink>,

    pub parent: Option<WeakLink>,
    pub first_child: Option<Link>,
    pub last_child: Option<WeakLink>,
    pub prev_sibling: Option<WeakLink>,
    pub next_sibling: Option<Link>,

    pub node_type: NodeType, // TODO: should be immutable/const somehow
    pub tag_name: Option<TagName>,
    pub id: String,
    pub attributes: Attributes,
    pub linked_nodes: Vec<WeakLink>,
    pub text: String,
}

impl NodeData {
    /// Detach a node from its parent and siblings. Children are not affected.
    pub fn detach(&mut self) {
        // TODO: detach doc

        let parent_weak = self.parent.take();
        let prev_weak = self.prev_sibling.take();
        let next_strong = self.next_sibling.take();

        let prev_opt = prev_weak.as_ref().and_then(|weak| weak.upgrade());

        if let Some(next) = next_strong.as_ref() {
            next.borrow_mut().prev_sibling = prev_weak;
        } else if let Some(parent) = parent_weak.as_ref() {
            if let Some(parent) = parent.upgrade() {
                parent.borrow_mut().last_child = prev_weak;
            }
        }

        if let Some(prev) = prev_opt {
            prev.borrow_mut().next_sibling = next_strong;
        } else if let Some(parent) = parent_weak.as_ref() {
            if let Some(parent) = parent.upgrade() {
                parent.borrow_mut().first_child = next_strong;
            }
        }
    }
}

impl Drop for NodeData {
    /// We have to remove nodes manually, to prevent reference circular references,
    /// which lead to memory leaks.
    fn drop(&mut self) {
        // Remove all children of the root node, aka Document.
        if self.node_type == NodeType::Root {
            // Root `Node` itself was already removed, so we have to
            // iterate over children.
            if let Some(child) = self.first_child.as_ref() {
                let mut root = Node(child.clone());
                while let Some(mut sibling) = root.next_sibling() {
                    sibling.remove();
                }

                root.remove();
            }
        }
    }
}
