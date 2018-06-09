// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    ListSeparator,
    ValueWriteOptions,
};

/// XML node indention
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Indent {
    /// Disable indention and new lines.
    None,
    /// Indent with spaces. Preferred range is 0..4.
    Spaces(u8),
    /// Indent with tabs.
    Tabs,
}

/// An attributes order.
///
/// Note: the `id` attribute is always first.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttributesOrder {
    /// Attributes are stored in the `Vec` and with this option,
    /// they will be written in the same order an in the `Vec`.
    AsIs,
    /// Write attributes in the alphabetical order.
    ///
    /// Only SVG attributes will be sorted. Non-SVG attributes will be written as-is.
    Alphabetical,
    /// Write attributes in the same order as they listed in the SVG spec.
    ///
    /// The current set of rules is pretty limited and doesn't follow the spec strictly.
    ///
    /// Only SVG attributes will be sorted. Non-SVG attributes will be written as-is.
    Specification,
}

/// Options that defines SVG writing.
#[derive(Debug)]
pub struct WriteOptions {
    /// Use single quote marks instead of double quote.
    ///
    /// # Examples
    ///
    /// Before:
    ///
    /// ```text
    /// <rect fill="red"/>
    /// ```
    ///
    /// After:
    ///
    /// ```text
    /// <rect fill='red'/>
    /// ```
    ///
    /// Default: disabled
    pub use_single_quote: bool,

    /// Set XML nodes indention.
    ///
    /// # Examples
    ///
    /// `Indent::None`
    ///
    /// Before:
    ///
    /// ```text
    /// <svg>
    ///     <rect fill="red"/>
    /// </svg>
    ///
    /// ```
    ///
    /// After:
    ///
    /// ```text
    /// <svg><rect fill="red"/></svg>
    /// ```
    ///
    /// Default: 4 spaces
    pub indent: Indent,

    /// Set XML attributes indention.
    ///
    /// # Examples
    ///
    /// `Indent::Spaces(2)`
    ///
    /// Before:
    ///
    /// ```text
    /// <svg>
    ///     <rect fill="red" stroke="black"/>
    /// </svg>
    ///
    /// ```
    ///
    /// After:
    ///
    /// ```text
    /// <svg>
    ///     <rect
    ///       fill="red"
    ///       stroke="black"/>
    /// </svg>
    /// ```
    ///
    /// Default: `None`
    pub attributes_indent: Indent,

    /// Set attributes order.
    ///
    /// Default: `AttributesOrder::Alphabetical`
    pub attributes_order: AttributesOrder,

    /// `svgtypes` options.
    pub values: ValueWriteOptions,
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions {
            indent: Indent::Spaces(4),
            attributes_indent: Indent::None,
            use_single_quote: false,
            attributes_order: AttributesOrder::Alphabetical,
            values: ValueWriteOptions {
                trim_hex_colors: false,
                remove_leading_zero: false,
                use_compact_path_notation: false,
                join_arc_to_flags: false,
                remove_duplicated_path_commands: false,
                use_implicit_lineto_commands: false,
                simplify_transform_matrices: false,
                list_separator: ListSeparator::Space,
            },
        }
    }
}
