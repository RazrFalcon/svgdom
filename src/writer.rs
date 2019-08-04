use std::fmt;

use log::warn;
use xmlwriter::XmlWriter;

use crate::{
    AttributeId,
    AttributeValue,
    Document,
    ElementId,
    FilterSvgAttrs,
    Node,
    NodeData,
    NodeEdge,
    NodeType,
    QName,
    WriteBuffer,
};

use crate::{
    ListSeparator,
    ValueWriteOptions,
};

pub use xmlwriter::Indent;


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


/// Writes a document into the buffer.
pub(crate) fn write_dom(doc: &Document, opt: &WriteOptions) -> String {
    let xml_opt = xmlwriter::Options {
        use_single_quote: opt.use_single_quote,
        indent: opt.indent,
        attributes_indent: opt.attributes_indent,
    };

    let mut xml = XmlWriter::new(xml_opt);
    for edge in doc.root().traverse() {
        match edge {
            NodeEdge::Start(node) => {
                match node.node_type() {
                    NodeType::Root => {}
                    NodeType::Element => {
                        match *node.tag_name() {
                            QName::Id(id) => xml.start_element(id.as_str()),
                            QName::Name(ref s) => xml.start_element(s),
                        }

                        write_attributes(&node, opt, &mut xml);

                        if node.has_tag_name(ElementId::Text) {
                            xml.set_preserve_whitespaces(true);
                        }
                    }
                    NodeType::Comment => {
                        xml.write_comment(&node.text());
                    }
                    NodeType::Text => {
                        xml.write_text(&node.text());
                    }
                }
            }
            NodeEdge::End(node) => {
                if node.is_element() {
                    xml.end_element();
                }

                if node.has_tag_name(ElementId::Text) {
                    xml.set_preserve_whitespaces(false);
                }
            }
        }
    }

    xml.end_document()
}

/// Writes attributes.
///
/// Order:
/// - 'id'
/// - sorted SVG attributes
/// - unsorted non-SVG attributes
fn write_attributes(
    node: &Node,
    opt: &WriteOptions,
    xml: &mut XmlWriter,
) {
    // Write root SVG node attributes.
    if node.has_tag_name(ElementId::Svg) {
        if node.parent().map(|v| v.is_root()) == Some(true) {
            xml.write_attribute("xmlns", "http://www.w3.org/2000/svg");

            let xlink_needed = node.descendants().any(|n| n.has_attribute(AttributeId::Href));
            if xlink_needed {
                xml.write_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
            }
        }
    }

    if node.has_id() {
        xml.write_attribute("id", &node.id())
    }

    let attrs = node.attributes();

    // sort attributes
    let mut ids: Vec<_> = attrs.iter().svg()
        .map(|(aid, attr)| (aid, attr.name.as_ref()))
        .collect();
    ids.sort_by_key(|&(x, _)| x as usize);

    for &(id, name) in &ids {
        let attr = attrs.get(name).unwrap();
        let name = match id {
            AttributeId::Href => "xlink:href",
            AttributeId::Space => "xml:space",
            _ => id.as_str(),
        };

        if id == AttributeId::Unicode {
            if let AttributeValue::String(ref s) = attr.value {
                xml.write_attribute_raw(name, |buf| write_escaped(s, buf));
            } else {
                warn!("An invalid 'unicode' attribute value: {:?}.", attr.value);
            }

        } else {
            xml.write_attribute_raw(name, |buf| attr.value.write_buf_opt(&opt.values, buf));
        }
    }

    // write non-SVG attributes
    for attr in attrs.iter() {
        if let QName::Name(ref name) = attr.name {
            xml.write_attribute_raw(name, |buf| attr.value.write_buf_opt(&opt.values, buf));
        }
    }
}

fn write_escaped(unicode: &str, out: &mut Vec<u8>) {
    use std::io::Write;

    if unicode.starts_with("&#") {
        out.extend_from_slice(unicode.as_bytes());
    } else {
        for c in unicode.chars() {
            out.extend_from_slice(b"&#x");
            write!(out, "{:x}", c as u32).unwrap();
            out.push(b';');
        }
    }
}

impl fmt::Debug for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            NodeType::Root => write!(f, "Root()"),
            NodeType::Element => {
                write!(f, "Element({}", self.tag_name)?;
                write_element_content(self, f, true, true)?;
                write!(f, ")")
            }
            NodeType::Comment => write!(f, "Comment({})", self.text),
            NodeType::Text => write!(f, "Text({})", self.text),
        }
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            NodeType::Root => write!(f, ""),
            NodeType::Element => {
                write!(f, "<{}", self.tag_name)?;
                write_element_content(self, f, true, false)?;
                write!(f, ">")
            }
            NodeType::Comment => write!(f, "<!--{}-->", self.text),
            NodeType::Text => write!(f, "{}", self.text),
        }
    }
}

fn write_element_content(
    node: &NodeData,
    f: &mut fmt::Formatter,
    space_before_attrs: bool,
    print_linked: bool,
) -> fmt::Result {
    if !node.id.is_empty() {
        write!(f, " id='{}'", node.id)?;
    }

    if !node.attributes.is_empty() {
        if space_before_attrs {
            write!(f, " ")?;
        }
        write!(f, "{}", node.attributes)?;
    }

    if print_linked && !node.linked_nodes.is_empty() {
        write!(f, "; linked-nodes:")?;
        for node in &node.linked_nodes {
            write!(f, " '{}'", *node.id())?;
        }
    }

    Ok(())
}
