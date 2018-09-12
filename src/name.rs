// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains a `Name` wrapper which is used for element tag name and attribute name.

use std::fmt;

use {
    AttributeId,
    ElementId,
    WriteBuffer,
    WriteOptions,
};

/// A trait for SVG id's.
pub trait SvgId: Copy + PartialEq {
    /// Converts ID into name.
    fn name(&self) -> &str;
}

impl SvgId for AttributeId {
    fn name(&self) -> &str { self.as_str() }
}

impl SvgId for ElementId {
    fn name(&self) -> &str { self.as_str() }
}

/// Qualified name.
#[derive(Clone, PartialEq, Debug)]
pub enum QName<T: SvgId> {
    /// For an SVG name.
    Id(T),
    /// For an unknown name.
    Name(String),
}

impl<T: SvgId> QName<T> {
    /// Returns `QName` as `QNameRef`.
    pub fn as_ref(&self) -> QNameRef<T> {
        match *self {
            QName::Id(id) => QNameRef::Id(id),
            QName::Name(ref name) => QNameRef::Name(name),
        }
    }

    /// Checks that this name has specified ID.
    pub fn has_id(&self, id: T) -> bool {
        match *self {
            QName::Id(id2) => id == id2,
            _ => false,
        }
    }
}

impl WriteBuffer for QName<AttributeId> {
    fn write_buf_opt(&self, _opt: &WriteOptions, buf: &mut Vec<u8>) {
        match *self {
            QName::Id(id) => {
                if self.has_id(AttributeId::Href) {
                    buf.extend_from_slice(b"xlink:");
                } else if self.has_id(AttributeId::Space) {
                    buf.extend_from_slice(b"xml:");
                }

                buf.extend_from_slice(id.name().as_bytes());
            }
            QName::Name(ref name) => {
                buf.extend_from_slice(name.as_bytes());
            }
        }
    }
}

impl WriteBuffer for QName<ElementId> {
    fn write_buf_opt(&self, _opt: &WriteOptions, buf: &mut Vec<u8>) {
        match *self {
            QName::Id(id) => {
                buf.extend_from_slice(id.name().as_bytes());
            }
            QName::Name(ref name) => {
                buf.extend_from_slice(name.as_bytes());
            }
        }
    }
}

impl fmt::Display for QName<AttributeId> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.with_write_opt(&WriteOptions::default()))
    }
}

impl fmt::Display for QName<ElementId> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.with_write_opt(&WriteOptions::default()))
    }
}

/// Qualified name reference.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum QNameRef<'a, T: SvgId> {
    /// For an SVG name.
    Id(T),
    /// For an unknown name.
    Name(&'a str),
}

impl<'a, T: SvgId> QNameRef<'a, T> {
    /// Checks that this name has specified ID.
    pub fn has_id(&self, id: T) -> bool {
        match *self {
            QNameRef::Id(id2) => id == id2,
            _ => false,
        }
    }
}

impl<'a, T: SvgId> From<T> for QNameRef<'a, T> {
    fn from(value: T) -> Self {
        QNameRef::Id(value.into())
    }
}

impl<'a, T: SvgId> From<&'a str> for QNameRef<'a, T> {
    fn from(value: &'a str) -> Self {
        QNameRef::Name(value.into())
    }
}

impl<'a, T: SvgId> From<QNameRef<'a, T>> for QName<T> {
    fn from(value: QNameRef<T>) -> Self {
        match value {
            QNameRef::Id(id) => QName::Id(id),
            QNameRef::Name(name) => QName::Name(name.into()),
        }
    }
}
