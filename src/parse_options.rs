// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ElementId;

// TODO: add strict option to treat warnings as errors
// TODO: option to disable path parsing
// TODO: set dpi for unit convert

/// Options used during parsing.
pub struct ParseOptions {
    /// Add comment nodes to DOM during parsing.
    pub parse_comments: bool,
    /// Add declaration nodes to DOM during parsing.
    pub parse_declarations: bool,
    /// Add unknown elements to DOM during parsing.
    ///
    /// All elements which is not defined in `ElementId` are unknown.
    pub parse_unknown_elements: bool,
    /// Add unknown attributes to elements during parsing.
    ///
    /// All attributes which is not defined in `AttributeId` are unknown.
    pub parse_unknown_attributes: bool,
    /// `px` unit in `<length>` is rudimentary, since it is the same as none.
    ///
    /// By default we parse it as is, but it can be disabled.
    pub parse_px_unit: bool,
    /// Skip specified SVG elements and all their children during parsing.
    pub skip_svg_elements: Vec<ElementId>,
}

impl Default for ParseOptions {
    fn default() -> ParseOptions {
        ParseOptions {
            parse_comments: true,
            parse_declarations: true,
            parse_unknown_elements: true,
            parse_unknown_attributes: true,
            parse_px_unit: true,
            skip_svg_elements: Vec::new(),
        }
    }
}
