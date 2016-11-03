// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// TODO: option to disable path parsing

/// Options that defines parsing.
pub struct ParseOptions {
    /// Add comment nodes to the DOM during parsing.
    pub parse_comments: bool,
    /// Add declaration nodes to the DOM during parsing.
    pub parse_declarations: bool,
    /// Add unknown elements to the DOM during parsing.
    ///
    /// All elements which is not defined in `ElementId` are unknown.
    pub parse_unknown_elements: bool,
    /// Add unknown attributes to elements during parsing.
    ///
    /// All attributes which is not defined in `AttributeId` are unknown.
    pub parse_unknown_attributes: bool,
    /// `px` unit in the `<length>` type is rudimentary, since it's the same as none.
    ///
    /// By default we parse it as is, but it can be disabled.
    pub parse_px_unit: bool,
    /// Skip unresolved references inside the `class` attribute.
    ///
    /// It's enabled by default, but if you disable it - all unresolved classes will be kept
    /// in the `class` attribute.
    pub skip_unresolved_classes: bool,
}

impl Default for ParseOptions {
    fn default() -> ParseOptions {
        ParseOptions {
            parse_comments: true,
            parse_declarations: true,
            parse_unknown_elements: true,
            parse_unknown_attributes: true,
            parse_px_unit: true,
            skip_unresolved_classes: true,
        }
    }
}
