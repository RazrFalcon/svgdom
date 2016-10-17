// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// List of supported node types.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum NodeType {
    /// Root node of the `Document`.
    ///
    /// Constructed with `Document`. Unavailable to user.
    Root,
    /// Element node.
    ///
    /// Only an element can have attributes, ID and tag name.
    Element,
    /// Declaration node.
    Declaration,
    /// Comment node.
    Comment,
    /// CDATA node.
    Cdata,
    /// Text node.
    Text,
}
