// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::Ref;

mod options;

pub use self::options::*;

use {
    Attribute,
    AttributeId,
    AttributeType,
    Document,
    ElementId,
    Name,
    Node,
    NodeEdge,
    NodeType,
    Traverse,
    WriteBuffer,
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

            if node.children().any(|c| c.node_type() == NodeType::Text) {
                write_text_elem(iter, depth, attrs_depth, opt, node, out);
                write_newline(opt.indent, out);
                return;
            }

            write_element_start(node, depth, attrs_depth, opt, out);

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

    match opt.attributes_order {
        AttributesOrder::AsIs => {
            for attr in attrs.iter() {
                write_attribute(attr, depth, attrs_depth, opt, out);
            }
        }
        AttributesOrder::Alphabetical => {
            // sort attributes
            let mut ids: Vec<AttributeId> = attrs.iter_svg().map(|(aid, _)| aid).collect();
            ids.sort_by_key(|x| *x as usize);

            for aid in &ids {
                let attr = attrs.get(*aid).unwrap();
                write_attribute(attr, depth, attrs_depth, opt, out);
            }

            // write non-SVG attributes
            for attr in attrs.iter() {
                if let Name::Name(_) = attr.name {
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }
        }
        AttributesOrder::Specification => {
            // sort attributes
            let mut ids: Vec<AttributeId> = attrs.iter_svg().map(|(aid, _)| aid).collect();
            ids.sort_by_key(|x| *x as usize);

            let mut ids2: Vec<AttributeId> = Vec::with_capacity(ids.len());

            // collect fill attributes
            for aid in &ids {
                if aid.is_fill() {
                    ids2.push(*aid);
                }
            }

            // collect stroke attributes
            for aid in &ids {
                if aid.is_stroke() {
                    ids2.push(*aid);
                }
            }

            // collect style attributes
            for aid in &ids {
                if aid.is_presentation() && !aid.is_fill() && !aid.is_stroke() {
                    ids2.push(*aid);
                }
            }

            // collect element-specific attributes
            if let Some(eid) = node.tag_id() {
                for aid in attrs_order_by_element(eid) {
                    if ids.contains(aid) {
                        ids2.push(*aid);
                    }
                }
            }

            // write sorted
            for aid in &ids2 {
                let attr = attrs.get(*aid).unwrap();
                write_attribute(attr, depth, attrs_depth, opt, out);
            }

            // write what is left
            for aid in &ids {
                if !ids2.contains(aid) {
                    let attr = attrs.get(*aid).unwrap();
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }

            // write non-SVG attributes
            for attr in attrs.iter() {
                if let Name::Name(_) = attr.name {
                    write_attribute(attr, depth, attrs_depth, opt, out);
                }
            }
        }
    }
}

static SVG_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Width,
    AttributeId::Height,
    AttributeId::ViewBox,
    AttributeId::PreserveAspectRatio,
    AttributeId::Version,
    AttributeId::BaseProfile,
];

static RECT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Width,
    AttributeId::Height,
    AttributeId::Rx,
    AttributeId::Ry,
];

static CIRCLE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::Cx,
    AttributeId::Cy,
    AttributeId::R,
];

static ELLIPSE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::Cx,
    AttributeId::Cy,
    AttributeId::Rx,
    AttributeId::Ry,
];

static LINE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::X1,
    AttributeId::Y1,
    AttributeId::X2,
    AttributeId::Y2,
];

static POLYLINE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::Points,
];

static PATH_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::D,
];

static USE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Width,
    AttributeId::Height,
    AttributeId::XlinkHref,
];

static IMAGE_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::PreserveAspectRatio,
    AttributeId::Transform,
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Width,
    AttributeId::Height,
    AttributeId::XlinkHref,
];

static TEXT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Transform,
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Dx,
    AttributeId::Dy,
    AttributeId::Rotate,
];

static TSPAN_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Dx,
    AttributeId::Dy,
    AttributeId::Rotate,
];

static LINEAR_GRADIENT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::X1,
    AttributeId::Y1,
    AttributeId::X2,
    AttributeId::Y2,
    AttributeId::GradientUnits,
    AttributeId::GradientTransform,
    AttributeId::SpreadMethod,
    AttributeId::XlinkHref,
];

static RADIAL_GRADIENT_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::Cx,
    AttributeId::Cy,
    AttributeId::R,
    AttributeId::Fx,
    AttributeId::Fy,
    AttributeId::GradientUnits,
    AttributeId::GradientTransform,
    AttributeId::SpreadMethod,
    AttributeId::XlinkHref,
];

static PATTERN_ATTRIBUTES: &'static [AttributeId] = &[
    AttributeId::ViewBox,
    AttributeId::X,
    AttributeId::Y,
    AttributeId::Width,
    AttributeId::Height,
    AttributeId::PatternUnits,
    AttributeId::PatternContentUnits,
    AttributeId::PatternTransform,
    AttributeId::XlinkHref,
];

fn attrs_order_by_element(eid: ElementId) -> &'static [AttributeId] {
    match eid {
        ElementId::Svg => SVG_ATTRIBUTES,
        ElementId::Rect => RECT_ATTRIBUTES,
        ElementId::Circle => CIRCLE_ATTRIBUTES,
        ElementId::Ellipse => ELLIPSE_ATTRIBUTES,
        ElementId::Line => LINE_ATTRIBUTES,
        ElementId::Polyline | ElementId::Polygon => POLYLINE_ATTRIBUTES,
        ElementId::Path => PATH_ATTRIBUTES,
        ElementId::Use => USE_ATTRIBUTES,
        ElementId::Image => IMAGE_ATTRIBUTES,
        ElementId::Text => TEXT_ATTRIBUTES,
        ElementId::Tspan => TSPAN_ATTRIBUTES,
        ElementId::LinearGradient => LINEAR_GRADIENT_ATTRIBUTES,
        ElementId::RadialGradient => RADIAL_GRADIENT_ATTRIBUTES,
        ElementId::Pattern => PATTERN_ATTRIBUTES,
        _ => &[],
    }
}

fn write_attribute(
    attr: &Attribute,
    depth: &Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    out: &mut Vec<u8>
) {
    if !opt.write_hidden_attributes && !attr.visible {
        return;
    }

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
    depth: &mut Depth,
    attrs_depth: &Depth,
    opt: &WriteOptions,
    root: &Node,
    out: &mut Vec<u8>,
) {
    for edge in iter {
        if let NodeEdge::End(node) = edge {
            if let NodeType::Element = node.node_type() {
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
                if child.text().trim().is_empty() {
                    if root.children().count() == 1 {
                        continue;
                    }
                }

                write_escaped_text(child.text().as_ref(), out);
            }
            _ => {
                warn!("'text' element should contain only element and text nodes");
            }
        }
    }

    write_element_end(&root, out);
}

fn write_escaped_text(text: &str, out: &mut Vec<u8>) {
    for c in text.as_bytes() {
        match *c {
            b'&'  => out.extend_from_slice(b"&amp;"),
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
