// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Options that defines SVG parsing.
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

    /// Skip attributes with invalid values.
    ///
    /// By default, attribute with an invalid value will lead to a parsing error.
    /// This flag allows converting this error into a warning.
    pub skip_invalid_attributes: bool,

    /// Skip invalid/unsupported CSS.
    ///
    /// By default, CSS with an invalid/unsupported value will lead to a parsing error.
    /// This flag allows converting this error into a warning.
    pub skip_invalid_css: bool,

    /// Ignore fallback value in paint attributes.
    ///
    /// If this option is enabled then the color part in attributes like this
    /// `fill="url(#lg1) #fff"` will be ignored.
    ///
    /// Otherwise `UnsupportedPaintFallback` error will occur during parsing.
    pub skip_paint_fallback: bool,

    /// Ignore elements crosslink.
    ///
    /// If this option is enabled then attributes that introduce crosslink will be skipped.
    ///
    /// Otherwise `ElementCrosslink` error will occur during parsing.
    pub skip_elements_crosslink: bool,
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
            skip_invalid_attributes: false,
            skip_invalid_css: false,
            skip_paint_fallback: false,
            skip_elements_crosslink: false,
        }
    }
}
