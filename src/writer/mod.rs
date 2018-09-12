// Copyright 2018 Evgeniy Reizner
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;

mod attrs_order;
mod options;

pub use self::options::*;
use self::attrs_order::attrs_order_by_element;

use {
    Attribute,
    AttributeId,
    AttributeType,
    Document,
    ElementId,
    FilterSvgAttrs,
    Node,
    NodeData,
    NodeEdge,
    NodeType,
    QName,
    Traverse,
};

/// A wrapper to use `fmt::Display` with `WriteOptions`.
///
/// Should be used via `WriteBuffer::with_write_opt`.
pub struct DisplaySvg<'a, T: 'a + WriteBuffer> {
    value: &'a T,
    opt: &'a WriteOptions,
}

impl<'a, T: WriteBuffer> fmt::Debug for DisplaySvg<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use Display.
        write!(f, "{}", self)
    }
}

impl<'a, T: WriteBuffer> fmt::Display for DisplaySvg<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        let mut out = Vec::with_capacity(32);
        self.value.write_buf_opt(self.opt, &mut out);
        write!(f, "{}", str::from_utf8(&out).unwrap())
    }
}


/// A trait for writing data to the buffer.
pub trait WriteBuffer {
    /// Writes data to the `Vec<u8>` buffer using specified `WriteOptions`.
    fn write_buf_opt(&self, opt: &WriteOptions, buf: &mut Vec<u8>);

    /// Writes data to the `Vec<u8>` buffer using default `WriteOptions`.
    fn write_buf(&self, buf: &mut Vec<u8>) {
        self.write_buf_opt(&WriteOptions::default(), buf);
    }

    /// Returns an object that implements `fmt::Display` using provided write options.
    fn with_write_opt<'a>(&'a self, opt: &'a WriteOptions) -> DisplaySvg<'a, Self>
        where Self: Sized
    {
        DisplaySvg { value: self, opt }
    }
}

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
}

/// Writes a document into the buffer.
pub(crate) fn write_dom(doc: &Document, opt: &WriteOptions, out: &mut Vec<u8>) {
    let mut depth = Depth::new(opt.indent);
    let mut attrs_depth = Depth::new(opt.attributes_indent);
    let mut iter = doc.root().traverse();

    attrs_depth.value += 1;

    while let Some(edge) = iter.next() {
        match edge {
            NodeEdge::Start(node) => {
                write_start_edge(
                    &node,
                    &mut iter,
                    &mut depth,
                    &attrs_depth,
                    opt,
                    out,
                )
            }
            NodeEdge::End(node) => {
                write_end_edge(&node, &mut depth, opt, out)
            }
        }
    }
}

fn is_text_node(node: &Node) -> bool {
       node.is_text()
    || node.is_tag_name(ElementId::Tspan)
    || node.is_tag_name(ElementId::Tref)
}

/// Writes node's start edge.
fn write_start_edge(
    node: &Node,
    iter: &mut Traverse<NodeData>,
    depth: &mut Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    match node.node_type() {
        NodeType::Root => {}
        NodeType::Element => {
            depth.write_indent(out);

            if node.children().any(|c| is_text_node(&c)) {
                write_text_elem(iter, depth, attrs_depth, opt, node, out);
                write_newline(opt.indent, out);
                return;
            }

            write_element_start(node, depth, attrs_depth, opt, out);

            if node.has_children() {
                let mut has_text = false;
                if let Some(c) = node.first_child() {
                    if c.is_text() {
                        has_text = true;
                    }
                }

                if !has_text {
                    depth.value += 1;
                    write_newline(opt.indent, out);
                }
            }
        }
        NodeType::Comment => {
            depth.write_indent(out);
            write_non_element_node(node, out);
            write_newline(opt.indent, out);
        }
        NodeType::Text => {
            write_non_element_node(node, out);
        }
    }
}

/// Writes a non element node.
///
/// Specifically: Declaration, Comment, Cdata and Text.
fn write_non_element_node(node: &Node, out: &mut Vec<u8>) {
    match node.node_type() {
        NodeType::Comment => {
            write_node(b"<!--", &node.text(), b"-->", out);
        }
        NodeType::Text => {
            write_escaped_text(node.text().as_ref(), out);
        }
        _ => unreachable!(),
    }
}

#[inline]
fn write_node(prefix: &[u8], data: &str, suffix: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(prefix);
    out.extend_from_slice(data.as_bytes());
    out.extend_from_slice(suffix);
}

/// Writes an element start.
///
/// Order:
/// - `<`
/// - tag name
/// - attributes
/// - closing tag, if a node has children
fn write_element_start(
    node: &Node,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    out.push(b'<');

    node.tag_name().write_buf_opt(opt, out);
    write_attributes(node, depth, attrs_depth, opt, out);

    if node.has_children() {
        out.push(b'>');
    }
}

/// Writes attributes.
///
/// Order:
/// - 'id'
/// - sorted SVG attributes
/// - unsorted non-SVG attributes
fn write_attributes(
    node: &Node,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    // Write root SVG node attributes.
    if node.is_tag_name(ElementId::Svg) {
        if node.parent().map(|v| v.is_root()) == Some(true) {
            let attr = Attribute::new("xmlns", "http://www.w3.org/2000/svg");
            write_attribute(&attr, depth, attrs_depth, opt, out);

            let xlink_needed = node.descendants().any(|n| n.has_attribute(AttributeId::Href));
            if xlink_needed {
                let attr = Attribute::new("xmlns:xlink", "http://www.w3.org/1999/xlink");
                write_attribute(&attr, depth, attrs_depth, opt, out);
            }
        }
    }

    // write 'id'
    if node.has_id() {
        let attr = Attribute::new(AttributeId::Id, node.id().clone());
        write_attribute(&attr, depth, attrs_depth, opt, out);
    }

    let attrs = node.attributes();

    match opt.attributes_order {
        AttributesOrder::AsIs => {
            for attr in attrs.iter() {
                write_attribute(attr, depth, attrs_depth, opt, out);
            }
        }
        AttributesOrder::Alphabetical => {
            // sort attributes
            let mut ids: Vec<_> = attrs.iter().svg().map(|(aid, attr)| (aid, attr.name.as_ref()))
                                       .collect();
            ids.sort_by_key(|&(x, _)| x as usize);

            for &(_, name) in &ids {
                let attr = attrs.get(name).unwrap();
                write_attribute(attr, depth, attrs_depth, opt, out);
            }

            // write non-SVG attributes
            for attr in attrs.iter() {
                if let QName::Name(_) = attr.name {
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }
        }
        AttributesOrder::Specification => {
            // sort attributes
            let mut ids: Vec<_> = attrs.iter().svg().map(|(aid, attr)| (aid, attr.name.as_ref()))
                                       .collect();
            ids.sort_by_key(|&(x, _)| x as usize);

            let mut ids2 = Vec::with_capacity(ids.len());

            // collect fill attributes
            for &(aid, name) in &ids {
                if aid.is_fill() {
                    ids2.push((aid, name));
                }
            }

            // collect stroke attributes
            for &(aid, name) in &ids {
                if aid.is_stroke() {
                    ids2.push((aid, name));
                }
            }

            // collect style attributes
            for &(aid, name) in &ids {
                if aid.is_presentation() && !aid.is_fill() && !aid.is_stroke() {
                    ids2.push((aid, name));
                }
            }

            // collect element-specific attributes
            if let Some(eid) = node.tag_id() {
                for name2 in attrs_order_by_element(eid) {
                    if ids.iter().any(|&(_, name)| name == *name2) {
                        ids2.push((AttributeId::X, *name2));
                    }
                }
            }

            // write sorted
            for &(_, name) in &ids2 {
                let attr = attrs.get(name).unwrap();
                write_attribute(attr, depth, attrs_depth, opt, out);
            }

            // write what is left
            for &(_, name) in &ids {
                if !ids2.iter().any(|&(_, name2)| name == name2) {
                    let attr = attrs.get(name).unwrap();
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }

            // write non-SVG attributes
            for attr in attrs.iter() {
                if let QName::Name(_) = attr.name {
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }
        }
    }
}

fn write_attribute(
    attr: &Attribute,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    if opt.attributes_indent == Indent::None {
        out.push(b' ');
    } else {
        out.push(b'\n');
        depth.write_indent(out);
        attrs_depth.write_indent(out);
    }

    attr.write_buf_opt(opt, out);
}

/// Writes a `text` element node and it's children.
fn write_text_elem(
    iter: &mut Traverse<NodeData>,
    depth: &mut Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    root: &Node,
    out: &mut Vec<u8>,
) {
    for edge in iter {
        if let NodeEdge::End(node) = edge {
            if node.is_element() {
                if node == *root {
                    break;
                }
            }
        }
    }

    _write_text_elem(root, depth, attrs_depth, opt, out);
}

fn _write_text_elem(
    root: &Node,
    depth: &mut Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>,
) {
    write_element_start(root, depth, attrs_depth, opt, out);

    for child in root.children() {
        match child.node_type() {
            NodeType::Element => {
                _write_text_elem(&child, depth, attrs_depth, opt, out);
            }
            NodeType::Text => {
                write_escaped_text(child.text().as_ref(), out);
            }
            _ => {
                warn!("'text' element should contain only element and text nodes");
            }
        }
    }

    write_element_end(&root, opt, out);
}

fn write_escaped_text(text: &str, out: &mut Vec<u8>) {
    for c in text.as_bytes() {
        match *c {
            b'&' => out.extend_from_slice(b"&amp;"),
            b'<' => out.extend_from_slice(b"&lt;"),
            b'>' => out.extend_from_slice(b"&gt;"),
            _    => out.push(*c),
        }
    }
}

/// Writes an element closing tag.
fn write_element_end(node: &Node, opt: &WriteOptions, out: &mut Vec<u8>) {
    if node.has_children() {
        out.extend_from_slice(b"</");
        node.tag_name().write_buf_opt(opt, out);
        out.push(b'>');
    } else {
        out.extend_from_slice(b"/>");
    }
}

/// Writes node's end edge.
fn write_end_edge(
    node: &Node,
    depth: &mut Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>,
) {
    if node.is_element() {
        if node.has_children() {
            if depth.value > 0 {
                depth.value -= 1;
            }
            depth.write_indent(out);
        }
        write_element_end(node, opt, out);
        write_newline(opt.indent, out);
    }
}

/// Writes a new line.
#[inline]
fn write_newline(indent: Indent, out: &mut Vec<u8>) {
    if indent != Indent::None {
        out.push(b'\n');
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
        write!(f, " id=\"{}\"", node.id)?;
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
            write!(f, " \"{}\"", *node.id())?;
        }
    }

    Ok(())
}
