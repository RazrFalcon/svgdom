// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use vec_map::{VecMap, Values};

use super::{Attribute, AttributeId, AttributeValue};

/// Wrapper around attributes list.
///
/// More low level API than in `Node`, but it supports getting a reference to the attribute,
/// and not only copy like `Node`'s API.
///
/// Use with care, since it didn't perform many check from `Node`'s API.
pub struct Attributes(VecMap<Attribute>);

impl Attributes {
    /// Constructs a new attribute.
    ///
    /// **Warning:** newer construct it manually. All nodes has `Attributes` by default.
    pub fn new() -> Attributes {
        Attributes(VecMap::new())
    }

    /// Returns a optional reference to `Attribute`.
    pub fn get(&self, id: AttributeId) -> Option<&Attribute> {
        self.0.get(id as usize)
    }

    /// Returns a optional mutable reference to `Attribute`.
    pub fn get_mut(&mut self, id: AttributeId) -> Option<&mut Attribute> {
        self.0.get_mut(id as usize)
    }

    /// Returns optional reference to `AttributeValue`.
    pub fn get_value(&self, id: AttributeId) -> Option<&AttributeValue> {
        self.0.get(id as usize).map(|x| &x.value)
    }

    /// Inserts new attribute. Previous will be overwritten.
    ///
    /// **Warning:** this method did not perform any checks for linked attributes.
    /// If you want to insert an linked attribute - use `Node::set_link_attribute()`.
    pub fn insert(&mut self, attr: Attribute) {
        self.0.insert(attr.id.clone() as usize, attr);
    }

    /// Removes an existing attribute.
    ///
    /// **Warning:** this method did not perform any checks for linked attributes.
    /// If you want to remove an linked attribute - use `Node::remove_attribute()`.
    pub fn remove(&mut self, id: AttributeId) {
        self.0.remove(id as usize);
    }

    /// Returns `true` if container contains an attribute such `id`.
    pub fn contains(&self, id: AttributeId) -> bool {
        self.0.contains_key(id as usize)
    }

    /// Returns an iterator over container values.
    pub fn values(&self) -> Values<Attribute> {
        self.0.values()
    }

    /// Returns count of the attributes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an existing attribute or `def_value`.
    pub fn get_or<'a>(&'a self, id: AttributeId, def_value: &'a AttributeValue) -> &AttributeValue {
        match self.get(id) {
            Some(a) => &a.value,
            None => def_value,
        }
    }
}
