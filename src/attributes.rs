// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::slice;
use std::mem;

use super::{Attribute, AttributeId, AttributeValue};

/// Wrapper around attributes list.
///
/// More low level API than in `Node`, but it supports getting a reference to the attribute,
/// and not only copy like `Node`'s API.
///
/// Use with care, since it didn't perform many check from `Node`'s API.
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    /// Constructs a new attribute.
    ///
    /// **Warning:** newer construct it manually. All nodes has `Attributes` by default.
    #[inline]
    pub fn new() -> Attributes {
        Attributes(Vec::new())
    }

    /// Returns a optional reference to `Attribute`.
    #[inline]
    pub fn get(&self, id: AttributeId) -> Option<&Attribute> {
        for v in &self.0 {
            if v.id == id {
                return Some(v);
            }
        }

        None
    }

    /// Returns a optional mutable reference to `Attribute`.
    #[inline]
    pub fn get_mut(&mut self, id: AttributeId) -> Option<&mut Attribute> {
        for v in &mut self.0 {
            if v.id == id {
                return Some(v);
            }
        }

        None
    }

    /// Returns optional reference to `AttributeValue`.
    #[inline]
    pub fn get_value(&self, id: AttributeId) -> Option<&AttributeValue> {
        for v in &self.0 {
            if v.id == id {
                return Some(&v.value);
            }
        }

        None
    }

    /// Returns optional mutable reference to `AttributeValue`.
    #[inline]
    pub fn get_value_mut(&mut self, id: AttributeId) -> Option<&mut AttributeValue> {
        for v in &mut self.0 {
            if v.id == id {
                return Some(&mut v.value);
            }
        }

        None
    }

    /// Returns an existing attribute or `def_value`.
    #[inline]
    pub fn get_value_or<'a>(&'a self, id: AttributeId, def_value: &'a AttributeValue)
                            -> &AttributeValue {
        match self.get(id) {
            Some(a) => &a.value,
            None => def_value,
        }
    }

    /// Inserts new attribute. Previous will be overwritten.
    ///
    /// **Warning:** this method did not perform any checks for linked attributes.
    /// If you want to insert an linked attribute - use `Node::set_link_attribute()`.
    pub fn insert(&mut self, attr: Attribute) {
        if self.0.capacity() == 0 {
            self.0.reserve(16);
        }

        let idx = self.0.iter().position(|x| x.id == attr.id);
        match idx {
            Some(i) => { mem::replace(&mut self.0[i], attr); }
            None => self.0.push(attr),
        }
    }

    /// Removes an existing attribute.
    ///
    /// **Warning:** this method did not perform any checks for linked attributes.
    /// If you want to remove an linked attribute - use `Node::remove_attribute()`.
    pub fn remove(&mut self, id: AttributeId) {
        let idx = self.0.iter().position(|x| x.id == id);
        if let Some(i) = idx {
            self.0.remove(i);
        }
    }

    /// Returns `true` if container contains an attribute such `id`.
    #[inline]
    pub fn contains(&self, id: AttributeId) -> bool {
        self.0.iter().any(|a| a.id == id)
    }

    /// Returns count of the attributes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if attributes list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator.
    #[inline]
    pub fn iter(&self) -> slice::Iter<Attribute> {
        self.0.iter()
    }

    /// Returns a mutable iterator.
    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<Attribute> {
        self.0.iter_mut()
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F>(&mut self, f: F)
        where F: FnMut(&Attribute) -> bool
    {
        self.0.retain(f)
    }
}
