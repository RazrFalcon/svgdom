// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use {Attributes, TagName};
use super::NodeType;

pub type Link = Rc<RefCell<NodeData>>;
pub type WeakLink = Weak<RefCell<NodeData>>;

pub struct NodeData {
    // TODO: check that doc is equal in append, insert, etc.
    pub doc: Option<WeakLink>,

    pub parent: Option<WeakLink>,
    pub first_child: Option<Link>,
    pub last_child: Option<WeakLink>,
    pub previous_sibling: Option<WeakLink>,
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
        // TODO: trim names
        // TODO: detach doc

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
