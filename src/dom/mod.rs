// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub use self::doc::Document;
pub use self::iterators::*;
pub use self::node::Node;
pub use self::node_type::NodeType;

use {Name, NameRef, ElementId};
/// Type alias for `NameRef<ElementId>`.
pub type TagNameRef<'a> = NameRef<'a, ElementId>;
/// Type alias for `Name<ElementId>`.
pub type TagName = Name<ElementId>;

mod doc;
mod iterators;
mod node;
mod node_data;
mod node_type;
