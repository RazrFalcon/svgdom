// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::Ref;

use {
    Attribute,
    AttributeId,
    Document,
    ElementId,
    Indent,
    Name,
    Node,
    NodeEdge,
    NodeType,
    Traverse,
    WriteBuffer,
    WriteOptions,
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
        let v = (self.value as i32 + i32::from(step)) as u32;
        for _ in 0..v {
            buf.extend_from_slice(&self.block);
        }
    }
}

/// Writes a document into the buffer.
pub fn write_dom(doc: &Document, opt: &WriteOptions, out: &mut Vec<u8>) {
    let mut depth = Depth::new(opt.indent);
    let mut attrs_depth = Depth::new(opt.attributes_indent);
    let mut iter = doc.root().traverse();

    attrs_depth.value += 1;

    while let Some(edge) = iter.next() {
        match edge {
            NodeEdge::Start(node) => {
                write_start_edge(&node, &mut iter, &mut depth, &attrs_depth, opt, out)
            }
            NodeEdge::End(node) => {
                write_end_edge(&node, &mut depth, opt.indent, out)
            }
        }
    }
}

/// Writes node's start edge.
fn write_start_edge(
    node: &Node,
    iter: &mut Traverse,
    depth: &mut Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    match node.node_type() {
        NodeType::Root => {}
        NodeType::Element => {
            depth.write_indent(out);
            write_element_start(node, depth, attrs_depth, opt, out);

            if node.is_tag_name(ElementId::Text) && node.has_children() {
                write_text_elem(iter, depth, attrs_depth, opt, node, out);
                write_newline(opt.indent, out);
                return;
            }

            if node.has_children() {
                let mut has_text = false;
                if let Some(c) = node.first_child() {
                    if c.node_type() == NodeType::Text {
                        has_text = true;
                    }
                }

                if !has_text {
                    depth.value += 1;
                    write_newline(opt.indent, out);
                }
            }
        }
        NodeType::Cdata => {
            depth.write_indent_with_step(-1, out);
            write_non_element_node(node, out);
            write_newline(opt.indent, out);
        }
        NodeType::Declaration |
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
        NodeType::Declaration => {
            write_node(b"<?xml ", &node.text(), b"?>", out);
        }
        NodeType::Comment => {
            write_node(b"<!--", &node.text(), b"-->", out);
        }
        NodeType::Cdata => {
            write_node(b"<![CDATA[", &node.text(), b"]]>", out);
        }
        NodeType::Text => {
            write_escaped_text(node.text().as_ref(), out);
        }
        _ => unreachable!(),
    }
}

#[inline]
fn write_node(prefix: &[u8], data: &Ref<String>, suffix: &[u8], out: &mut Vec<u8>) {
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

    write_tag_name(&node.tag_name().unwrap(), out);
    write_attributes(node, depth, attrs_depth, opt, out);

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
fn write_attributes(
    node: &Node,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    // write 'id'
    if node.has_id() {
        let attr = Attribute::new(AttributeId::Id, node.id().clone());
        write_attribute(&attr, depth, attrs_depth, opt, out);
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

        write_attribute(attr, depth, attrs_depth, opt, out);
    }

    // write non-SVG attributes
    for attr in attrs.iter() {
        if let Name::Name(_) = attr.name {
            write_attribute(attr, depth, attrs_depth, opt, out);
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
    iter: &mut Traverse,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    root: &Node,
    out: &mut Vec<u8>
) {
    for edge in iter {
        match edge {
            NodeEdge::Start(node) => {
                match node.node_type() {
                    NodeType::Element => {
                        write_element_start(&node, depth, attrs_depth, opt, out);
                    }
                    NodeType::Text => {
                        write_escaped_text(node.text().as_ref(), out);
                    }
                    _ => {}
                }
            }
            NodeEdge::End(node) => {
                if let NodeType::Element = node.node_type() {
                    if node == *root {
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

fn write_escaped_text(text: &str, out: &mut Vec<u8>) {
    for c in text.as_bytes() {
        match *c {
            b'"'  => out.extend_from_slice(b"&quot;"),
            b'&'  => out.extend_from_slice(b"&amp;"),
            b'\'' => out.extend_from_slice(b"&apos;"),
            b'<'  => out.extend_from_slice(b"&lt;"),
            b'>'  => out.extend_from_slice(b"&gt;"),
            _     => out.push(*c),
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
    if let NodeType::Element = node.node_type() {
        if node.has_children() {
            if depth.value > 0 {
                depth.value -= 1;
            }
            depth.write_indent(out);
        }
        write_element_end(node, out);
        write_newline(indent, out);
    }
}

/// Writes a new line.
#[inline]
fn write_newline(indent: Indent, out: &mut Vec<u8>) {
    if indent != Indent::None {
        out.push(b'\n');
    }
}
