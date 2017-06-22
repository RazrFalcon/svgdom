// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains a `Name` wrapper which is used for element tag name and attribute name.

use std::fmt;

/// A trait for SVG id's.
pub trait SvgId: Copy {
    /// Converts ID into name.
    fn name(&self) -> &str;
}

// TODO: try Cow

/// A container for an SVG item name.
#[derive(Clone,PartialEq)]
pub enum Name<T: SvgId> {
    /// For an SVG name.
    Id(T),
    /// For an unknown name.
    Name(String),
}

/// A reference-like container for a [`Name`] object.
///
/// We need this to prevent `String` copy.
///
/// [`Name`]: enum.Name.html
#[derive(Clone,Copy,PartialEq)]
pub enum NameRef<'a, T: SvgId> {
    /// For an SVG name.
    Id(T),
    /// For an unknown name.
    Name(&'a str),
}

impl<'a, T: SvgId> From<T> for NameRef<'a, T> {
    fn from(value: T) -> NameRef<'a, T> {
        NameRef::Id(value)
    }
}

impl<'a, T: SvgId> From<&'a str> for NameRef<'a, T> {
    fn from(value: &'a str) -> NameRef<'a, T> {
        NameRef::Name(value)
    }
}

impl<'a, T: SvgId> From<NameRef<'a, T>> for Name<T> {
    fn from(value: NameRef<T>) -> Name<T> {
        match value {
            NameRef::Id(id) => Name::Id(id),
            NameRef::Name(name) => Name::Name(name.to_string()),
        }
    }
}

impl<'a, T: SvgId> fmt::Debug for NameRef<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NameRef::Id(id) => write!(f, "{}", id.name()),
            NameRef::Name(name) => write!(f, "{}", name),
        }
    }
}

impl<T: SvgId> Name<T> {
    /// Converts `Name` into `NameRef`.
    pub fn into_ref(&self) -> NameRef<T> {
        match *self {
            Name::Id(id) => NameRef::Id(id),
            Name::Name(ref name) => NameRef::Name(name),
        }
    }
}

// TODO: add Display
impl<T: SvgId> fmt::Debug for Name<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.into_ref())
    }
}
