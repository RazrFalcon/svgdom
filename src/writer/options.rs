use crate::{
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

    /// `svgtypes` options.
    pub values: ValueWriteOptions,
}

impl Default for WriteOptions {
    fn default() -> Self {
        WriteOptions {
            indent: Indent::Spaces(4),
            attributes_indent: Indent::None,
            use_single_quote: false,
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
