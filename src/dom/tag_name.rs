// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use ElementId;

// TODO: maybe join with AttributeName(Ref), since they are the same

/// A reference-like container for a `TagName` object.
///
/// We need this to prevent `String` copy.
#[derive(Clone,Copy,PartialEq)]
pub enum TagNameRef<'a> {
    /// For SVG elements.
    Id(ElementId),
    /// For unknown elements.
    Name(&'a str),
}

impl<'a> From<ElementId> for TagNameRef<'a> {
    fn from(value: ElementId) -> TagNameRef<'a> {
        TagNameRef::Id(value)
    }
}

impl<'a> From<&'a str> for TagNameRef<'a> {
    fn from(value: &'a str) -> TagNameRef<'a> {
        TagNameRef::Name(value)
    }
}

/// A container for an element tag name.
#[derive(Clone,PartialEq)]
pub enum TagName {
    /// For SVG elements.
    Id(ElementId),
    /// For unknown elements.
    Name(String),
}

impl<'a> From<TagNameRef<'a>> for TagName {
    fn from(value: TagNameRef) -> TagName {
        match value {
            TagNameRef::Id(id) => TagName::Id(id),
            TagNameRef::Name(ref name) => TagName::Name(name.to_string()),
        }
    }
}

impl<'a> fmt::Debug for TagNameRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TagNameRef::Id(ref id) => write!(f, "{}", id.name()),
            TagNameRef::Name(ref name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Debug for TagName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.into_ref())
    }
}

impl TagName {
    /// Converts `TagName` into `TagNameRef`.
    pub fn into_ref(&self) -> TagNameRef {
        match *self {
            TagName::Id(id) => TagNameRef::Id(id),
            TagName::Name(ref name) => TagNameRef::Name(name),
        }
    }
}
