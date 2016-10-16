// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::str::FromStr;

use ElementId;

/// Wrapper arrow element tag name.
#[derive(Clone,PartialEq)]
pub enum TagName {
    /// For SVG elements.
    Id(ElementId),
    /// For unknown elements.
    Name(String),
}

impl From<ElementId> for TagName {
    fn from(value: ElementId) -> TagName {
        TagName::Id(value)
    }
}

impl From<String> for TagName {
    fn from(value: String) -> TagName {
        TagName::Name(value)
    }
}

impl<'a> From<&'a str> for TagName {
    fn from(value: &str) -> TagName {
        TagName::Name(String::from_str(value).unwrap())
    }
}

impl fmt::Debug for TagName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TagName::Id(ref id) => write!(f, "{}", id.name()),
            TagName::Name(ref name) => write!(f, "{}", name),
        }
    }
}
