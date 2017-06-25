// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem;
use std::iter::{
    Filter,
    Map,
};
use std::slice::{
    Iter,
    IterMut,
};

use {
    Attribute,
    AttributeId,
    AttributeNameRef,
    AttributeValue,
};

// TODO: bench with HashTable
// TODO: iter_svg() -> iter().svg() like in dom iterators

/// Filter iterator over SVG attributes.
pub type SvgAttrFilter<'a> = Filter<Iter<'a, Attribute>, fn(&&Attribute) -> bool>;
/// Mutable filter iterator over SVG attributes.
pub type SvgAttrFilterMut<'a> = Filter<IterMut<'a, Attribute>, fn(&&mut Attribute) -> bool>;

/// Wrapper around attributes list.
///
/// More low level API than in [`Node`], but it supports getting a reference to the attribute,
/// and not a copy.
///
/// [`Node`]: struct.Node.html
pub struct Attributes(Vec<Attribute>);

impl Attributes {
    /// Constructs a new attribute.
    ///
    /// **Warning:** this method is for private use only. Never invoke it directly.
    #[inline]
    pub fn new() -> Attributes {
        Attributes(Vec::new())
    }

    /// Returns an optional reference to [`Attribute`].
    ///
    /// [`Attribute`]: struct.Attribute.html
    #[inline]
    pub fn get<'a, N>(&self, name: N) -> Option<&Attribute>
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        for v in &self.0 {
            if v.name.into_ref() == name {
                return Some(v);
            }
        }

        None
    }

    /// Returns an optional mutable reference to [`Attribute`].
    ///
    /// [`Attribute`]: struct.Attribute.html
    #[inline]
    pub fn get_mut<'a, N>(&mut self, name: N) -> Option<&mut Attribute>
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        for v in &mut self.0 {
            if v.name.into_ref() == name {
                return Some(v);
            }
        }

        None
    }

    /// Returns an optional reference to [`AttributeValue`].
    ///
    /// [`AttributeValue`]: enum.AttributeValue.html
    #[inline]
    pub fn get_value<'a, N>(&self, name: N) -> Option<&AttributeValue>
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        for v in &self.0 {
            if v.name.into_ref() == name {
                return Some(&v.value);
            }
        }

        None
    }

    /// Returns an optional mutable reference to [`AttributeValue`].
    ///
    /// [`AttributeValue`]: enum.AttributeValue.html
    #[inline]
    pub fn get_value_mut<'a, N>(&mut self, name: N) -> Option<&mut AttributeValue>
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        for v in &mut self.0 {
            if v.name.into_ref() == name {
                return Some(&mut v.value);
            }
        }

        None
    }

    /// Inserts a new attribute. Previous will be overwritten.
    ///
    /// # Panics
    ///
    /// During insert of a linked attribute. Use [`Node::set_attribute()`] instead.
    ///
    /// Will panic only in debug build.
    ///
    /// [`Node::set_attribute()`]: struct.Node.html#method.set_attribute
    pub fn insert(&mut self, attr: Attribute) {
        if cfg!(debug_assertions) {
            if attr.is_link() || attr.is_func_link() {
                panic!("attribute with Link/FuncLink value must be set only via Node::set_attribute");
            }
        }

        self.insert_impl(attr);
    }

    /// Creates a new attribute from name and value and inserts it. Previous will be overwritten.
    ///
    /// [`Node`] attribute value can be set only via [`Node::set_attribute()`] method.
    ///
    /// [`Node`]: struct.Node.html
    /// [`Node::set_attribute()`]: struct.Node.html#method.set_attribute
    pub fn insert_from<'a, N, T>(&mut self, name: N, value: T)
        where AttributeNameRef<'a>: From<N>, AttributeValue: From<T>
    {
        self.insert(Attribute::new(name, value));
    }

    /// Inserts a new link attribute.
    ///
    /// **Warning:** this method is for private use only. Never invoke it directly.
    pub fn insert_impl(&mut self, attr: Attribute) {
        // Increase capacity on first insert.
        if self.0.capacity() == 0 {
            self.0.reserve(16);
        }

        let idx = self.0.iter().position(|x| x.name == attr.name);
        match idx {
            // We use braces to discard return value.
            Some(i) => { mem::replace(&mut self.0[i], attr); }
            None => self.0.push(attr),
        }
    }

    /// Removes an existing attribute.
    ///
    /// # Panics
    ///
    /// During remove of a linked attribute. Use [`Node::remove_attribute()`] instead.
    ///
    /// Will panic only in debug build.
    ///
    /// [`Node::remove_attribute()`]: struct.Node.html#method.remove_attribute
    pub fn remove<'a, N>(&mut self, name: N)
        where AttributeNameRef<'a>: From<N>, N: Copy
    {
        // Checks that removed attribute is not linked.
        //
        // Since this check is expensive - we do it only in debug build.
        if cfg!(debug_assertions) {
            let name = AttributeNameRef::from(name);
            let attr = self.0.iter().find(|x| x.name.into_ref() == name);
            if let Some(attr) = attr {
                if attr.is_link() || attr.is_func_link() {
                    panic!("attribute with Link/FuncLink value must be remove \
                            only via Node::remove_attribute");
                }
            }
        }

        self.remove_impl(name);
    }

    /// Removes an existing attribute.
    ///
    /// **Warning:** this method is for private use only. Never invoke it directly.
    pub fn remove_impl<'a, N>(&mut self, name: N)
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        let idx = self.0.iter().position(|x| x.name.into_ref() == name);
        if let Some(i) = idx {
            self.0.remove(i);
        }
    }

    /// Returns `true` if the container contains an attribute with such `id`.
    #[inline]
    pub fn contains<'a, N>(&self, name: N) -> bool
        where AttributeNameRef<'a>: From<N>
    {
        let name = AttributeNameRef::from(name);
        self.0.iter().any(|a| a.name.into_ref() == name)
    }

    /// Returns count of the attributes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if attributes is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator.
    #[inline]
    pub fn iter(&self) -> Iter<Attribute> {
        self.0.iter()
    }

    /// Returns a mutable iterator.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<Attribute> {
        self.0.iter_mut()
    }

    /// Returns an iterator over SVG attributes.
    ///
    /// Shorthand for: `iter().filter(|a| a.is_svg()).map(|a| (a.id().unwrap(), a))`
    #[inline]
    pub fn iter_svg<'a>(&'a self)
        -> Map<SvgAttrFilter, fn(&'a Attribute) -> (AttributeId, &'a Attribute)>
    {
        fn map_svg(a: &Attribute) -> (AttributeId, &Attribute) { (a.id().unwrap(), a) }
        self.filter_svg().map(map_svg)
    }

    /// Returns a mutable iterator over SVG attributes.
    ///
    /// Shorthand for: `iter_mut().filter(|a| a.is_svg()).map(|a| (a.id().unwrap(), a))`
    #[inline]
    pub fn iter_svg_mut<'a>(&'a mut self)
        -> Map<SvgAttrFilterMut, fn(&'a mut Attribute) -> (AttributeId, &'a mut Attribute)>
    {
        fn map_svg(a: &mut Attribute) -> (AttributeId, &mut Attribute)
        { (a.id().unwrap(), a) }

        self.filter_svg_mut().map(map_svg)
    }

    #[inline]
    fn filter_svg(&self) -> SvgAttrFilter {
        fn is_svg(a: &&Attribute) -> bool { a.is_svg() }
        self.iter().filter(is_svg)
    }

    #[inline]
    fn filter_svg_mut(&mut self) -> SvgAttrFilterMut {
        fn is_svg(a: &&mut Attribute) -> bool { a.is_svg() }
        self.iter_mut().filter(is_svg)
    }

    /// Retains only elements specified by the predicate.
    ///
    /// # Panics
    ///
    /// During remove of a linked attribute. Use [`Node::remove_attribute()`] instead.
    ///
    /// Will panic only in debug build.
    ///
    /// [`Node::remove_attribute()`]: struct.Node.html#method.remove_attribute
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&Attribute) -> bool
    {
        // Checks that removed attribute is not linked.
        //
        // Since this check is expensive - we do it only in debug build.
        if cfg!(debug_assertions) {
            for attr in &self.0 {
                if !f(attr) {
                    if attr.is_link() || attr.is_func_link() {
                        panic!("attribute with Link/FuncLink value must be remove \
                                only via Node::remove_attribute");
                    }
                }
            }
        }

        self.0.retain(f)
    }
}
