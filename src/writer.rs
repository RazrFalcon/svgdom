// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module contains a default implementation of the SVG writer.

use std::cell::Ref;

use super::{
    Attribute,
    AttributeId,
    Document,
    ElementId,
    Node,
    NodeEdge,
    NodeType,
    Name,
    Traverse,
    WriteBuffer,
    WriteOptions,
    Indent,
};

/// An indent counter.
struct Depth {
    /// Current depth.
    value: u32,
    block: Vec<u8>,
}

impl Depth {
    /// Creates a new `Depth`.
    #[inline]
    fn new(indent: Indent) -> Depth {
        Depth {
            value: 0,
            block: Depth::gen_indent(indent),
        }
    }

    fn gen_indent(indent: Indent) -> Vec<u8> {
        match indent {
            Indent::None => Vec::new(),
            Indent::Spaces(n) => {
                let mut v = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    v.push(b' ');
                }
                v
            }
            Indent::Tabs => vec![b'\t'],
        }
    }

    /// Writes an indent to the buffer.
    #[inline]
    fn write_indent(&self, buf: &mut Vec<u8>) {
        for _ in 0..self.value {
            buf.extend_from_slice(&self.block);
        }
    }

    /// Writes a relative indent to the buffer.
    #[inline]
    fn write_indent_with_step(&self, step: i8, buf: &mut Vec<u8>) {
        let v = (self.value as i32 + step as i32) as u32;
        for _ in 0..v {
            buf.extend_from_slice(&self.block);
        }
    }
}

/// Writes a document into the buffer.
pub fn write_dom(doc: &Document, opt: &WriteOptions, out: &mut Vec<u8>) {
    let mut depth = Depth::new(opt.indent);
    let mut iter = doc.root().traverse();

    while let Some(edge) = iter.next() {
        match edge {
            NodeEdge::Start(node) => {
                write_start_edge(&node, &mut iter, &mut depth, opt, out)
            }
            NodeEdge::End(node) => {
                write_end_edge(&node, &mut depth, opt.indent, out)
            }
        }
    }
}

/// Writes node's start edge.
fn write_start_edge(node: &Node, iter: &mut Traverse, depth: &mut Depth, opt: &WriteOptions,
                    out: &mut Vec<u8>) {
    match node.node_type() {
        NodeType::Root => {}
        NodeType::Element => {
            depth.write_indent(out);
            write_element_start(&node, opt, out);

            if node.is_tag_name(ElementId::Text) && node.has_children() {
                write_text_elem(iter, opt, &node, &depth, out);
                write_newline(opt.indent, out);
                return;
            }

            if node.has_children() {
                depth.value += 1;
                write_newline(opt.indent, out);
            }
        }
        NodeType::Declaration => {
            depth.write_indent(out);
            write_non_element_node(&node, out);
            write_newline(opt.indent, out);
        }
        NodeType::Comment => {
            depth.write_indent(out);
            write_non_element_node(&node, out);
            write_newline(opt.indent, out);
        }
        NodeType::Cdata => {
            depth.write_indent_with_step(-1, out);
            write_non_element_node(&node, out);
            write_newline(opt.indent, out);
        }
        NodeType::Text => {
            // TODO: implement xml escape
            depth.write_indent(out);
            write_non_element_node(&node, out);
            write_newline(opt.indent, out);
        }
    }
}

/// Writes a non element node.
///
/// Specifically: Declaration, Comment, Cdata and Text.
fn write_non_element_node(node: &Node, out: &mut Vec<u8>) {
    match node.node_type() {
        NodeType::Declaration => {
            write_node(b"<?xml ", node.text(), b"?>", out);
        }
        NodeType::Comment => {
            write_node(b"<!--", node.text(), b"-->", out);
        }
        NodeType::Cdata => {
            write_node(b"<![CDATA[", node.text(), b"]]>", out);
        }
        NodeType::Text => {
            // TODO: implement xml escape
            out.extend_from_slice(node.text().unwrap().trim().as_bytes());
        }
        _ => unreachable!(),
    }
}

#[inline]
fn write_node(prefix: &[u8], data: Option<Ref<String>>, suffix: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(prefix);
    out.extend_from_slice(data.unwrap().as_bytes());
    out.extend_from_slice(suffix);
}

/// Writes an element start.
///
/// Order:
/// - `<`
/// - tag name
/// - attributes
/// - closing tag, if a node has children
fn write_element_start(node: &Node, opt: &WriteOptions, out: &mut Vec<u8>) {
    out.push(b'<');

    write_tag_name(&node.tag_name().unwrap(), out);
    write_attributes(node, opt, out);

    if node.has_children() {
        out.push(b'>');
    }
}

/// Writes an element tag name.
fn write_tag_name(tag_name: &Name<ElementId>, out: &mut Vec<u8>) {
    match *tag_name {
        Name::Id(ref id) => {
            out.extend_from_slice(id.name().as_bytes());
        }
        Name::Name(ref name) => {
            let n = name.clone();
            out.extend_from_slice(n.as_bytes());
        }
    }
}

/// Writes attributes.
///
/// Order:
/// - 'id'
/// - sorted SVG attributes
/// - unsorted non-SVG attributes
fn write_attributes(node: &Node, opt: &WriteOptions, out: &mut Vec<u8>) {
    // write 'id'
    if node.has_id() {
        out.push(b' ');
        let attr = Attribute::new(AttributeId::Id, node.id().clone());
        attr.write_buf_opt(opt, out);
    }

    let attrs = node.attributes();

    // write sorted SVG attributes

    // TODO: make optional
    // sort attributes
    let mut ids: Vec<AttributeId> = attrs.iter_svg().map(|(aid, _)| aid).collect();
    ids.sort_by_key(|x| *x as usize);

    for aid in &ids {
        let attr = attrs.get(*aid).unwrap();

        if !opt.write_hidden_attributes && !attr.visible {
            continue;
        }

        out.push(b' ');
        attr.write_buf_opt(opt, out);
    }

    // write non-SVG attributes
    for attr in attrs.iter() {
        if let Name::Name(_) = attr.name {
            out.push(b' ');
            attr.write_buf_opt(opt, out);
        }
    }
}

/// Writes a `text` element node and it's children.
///
/// This method will check for the `xml:space` attribute and will properly write indented text.
fn write_text_elem(iter: &mut Traverse, opt: &WriteOptions, root: &Node, depth: &Depth,
                    out: &mut Vec<u8>) {
    let is_root_preserve = root.has_attribute_with_value(AttributeId::XmlSpace, "preserve");

    if !is_root_preserve {
        write_newline(opt.indent, out);
        depth.write_indent_with_step(1, out);
    }

    // Check that 'text' element contains only one text node.
    // We use 2, since 'descendants' includes current node.
    let is_simple_text = root.descendants().count() == 2;

    let mut is_first_text = true;

    for edge in iter {
        match edge {
            NodeEdge::Start(node) => {
                match node.node_type() {
                    NodeType::Element => {
                        write_element_start(&node, opt, out);
                    }
                    NodeType::Text => {
                        if is_root_preserve {
                            out.extend_from_slice(node.text().unwrap().as_bytes());
                        } else if is_simple_text {
                            out.extend_from_slice(node.text().unwrap().trim().as_bytes());
                        } else if is_first_text {
                            out.extend_from_slice(node.text().unwrap().trim_left().as_bytes());
                        } else if root.last_child().unwrap() == node {
                            out.extend_from_slice(node.text().unwrap().trim_right().as_bytes());
                        } else {
                            out.extend_from_slice(node.text().unwrap().as_bytes());
                        }
                        is_first_text = false;
                    }
                    _ => {}
                }
            }
            NodeEdge::End(node) => {
                if let NodeType::Element = node.node_type() {
                    if node == *root {
                        if !is_root_preserve {
                            write_newline(opt.indent, out);
                            depth.write_indent(out);
                        }
                        write_element_end(&node, out);

                        break;
                    } else {
                        write_element_end(&node, out);
                    }
                }
            }
        }
    }
}

/// Writes an element closing tag.
fn write_element_end(node: &Node, out: &mut Vec<u8>) {
    if node.has_children() {
        out.extend_from_slice(b"</");
        write_tag_name(&node.tag_name().unwrap(), out);
        out.push(b'>');
    } else {
        out.extend_from_slice(b"/>");
    }
}

/// Writes node's end edge.
fn write_end_edge(node: &Node, depth: &mut Depth, indent: Indent, out: &mut Vec<u8>) {
    match node.node_type() {
        NodeType::Element => {
            if node.has_children() {
                if depth.value > 0 {
                    depth.value -= 1;
                }
                depth.write_indent(out);
            }
            write_element_end(&node, out);
            write_newline(indent, out);
        }
        _ => {}
    }
}

/// Writes a new line.
#[inline]
fn write_newline(indent: Indent, out: &mut Vec<u8>) {
    if indent != Indent::None {
        out.push(b'\n');
    }
}
