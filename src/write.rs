// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{
    AttributeId,
    Document,
    ElementId,
    Node,
    NodeEdge,
    NodeType,
    TagName,
    Traverse,
    WriteBuffer,
    WriteOptions,
};

struct Depth {
    value: u32,
    block: Vec<u8>,
}

impl Depth {
    fn new(indent: i8) -> Depth {
        Depth {
            value: 0,
            block: Depth::gen_indent(indent),
        }
    }

    fn gen_indent(len: i8) -> Vec<u8> {
        match len {
            -1...0 => Vec::new(),
            _ => {
                let mut v = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    v.push(b' ');
                }
                v
            }
        }
    }

    fn write_indent(&self, s: &mut Vec<u8>) {
        for _ in 0..self.value {
            s.extend_from_slice(&self.block);
        }
    }

    fn write_indent_with_step(&self, step: i8, s: &mut Vec<u8>) {
        let v = (self.value as i32 + step as i32) as u32;
        for _ in 0..v {
            s.extend_from_slice(&self.block);
        }
    }
}

pub fn write_dom(doc: &Document, opt: &WriteOptions, out: &mut Vec<u8>) {
    let mut depth = Depth::new(opt.indent);

    let mut iter = doc.root().traverse();

    while let Some(edge) = iter.next() {
        match edge {
            NodeEdge::Start(node) => {
                match node.node_type() {
                    NodeType::Root => continue,
                    NodeType::Element => {

                        depth.write_indent(out);

                        if node.is_tag_name(&TagName::Id(ElementId::Text)) && node.has_children() {
                            write_element_start(&node, opt, out);
                            process_text(&mut iter, opt, &node, &depth, out);
                            write_newline(opt.indent, out);
                            continue;
                        }

                        write_element_start(&node, opt, out);

                        if node.has_children() {
                            depth.value += 1;
                            write_newline(opt.indent, out);
                        }
                    },
                    NodeType::Declaration => {
                        depth.write_indent(out);
                        out.extend_from_slice(b"<?xml ");
                        out.extend_from_slice(node.text().unwrap().as_bytes());
                        out.extend_from_slice(b"?>");
                        write_newline(opt.indent, out);
                    },
                    NodeType::Comment => {
                        depth.write_indent(out);
                        out.extend_from_slice(b"<!--");
                        out.extend_from_slice(node.text().unwrap().as_bytes());
                        out.extend_from_slice(b"-->");
                        write_newline(opt.indent, out);
                    },
                    NodeType::Cdata => {
                        depth.write_indent_with_step(-1, out);
                        out.extend_from_slice(b"<![CDATA[");
                        out.extend_from_slice(node.text().unwrap().as_bytes());
                        out.extend_from_slice(b"]]>");
                        write_newline(opt.indent, out);
                    },
                    NodeType::Text => {
                        // TODO: implement xml escape
                        depth.write_indent(out);
                        out.extend_from_slice(node.text().unwrap().trim().as_bytes());
                        write_newline(opt.indent, out);
                    },
                }
            },
            NodeEdge::End(node) => {
                match node.node_type() {
                    NodeType::Root => continue,
                    NodeType::Element => {
                        if node.has_children() {
                            if depth.value > 0 {
                                depth.value -= 1;
                            }
                            depth.write_indent(out);
                        }
                        write_element_end(&node, out);
                        write_newline(opt.indent, out);
                    },
                    _ => {},
                }
            },
        }
    }
}

fn write_quote(opt: &WriteOptions, out: &mut Vec<u8>) {
    if opt.use_single_quote {
        out.push(b'\'');
    } else {
        out.push(b'"');
    }
}

fn write_attribute_name(name: &[u8], opt: &WriteOptions, out: &mut Vec<u8>) {
    out.extend_from_slice(name);
    out.push(b'=');
    write_quote(opt, out);
}

fn write_attribute(name: &[u8], value: &[u8], opt: &WriteOptions, out: &mut Vec<u8>) {
    write_attribute_name(name, opt, out);
    out.extend_from_slice(value);
    write_quote(opt, out);
}

fn write_tag_name(tag_name: &TagName, out: &mut Vec<u8>) {
    match *tag_name {
        TagName::Id(ref id) => {
            out.extend_from_slice(id.name().as_bytes());
        }
        TagName::Name(ref name) => {
            let n = name.clone();
            out.extend_from_slice(n.as_bytes());
        }
    }
}

fn write_newline(indent: i8, out: &mut Vec<u8>) {
    if indent >= 0 {
        out.push(b'\n');
    }
}

fn write_attributes(node: &Node, opt: &WriteOptions, out: &mut Vec<u8>) {
    let id = node.id();
    if !id.is_empty() {
        let idvec = &id[..];
        out.push(b' ');
        write_attribute(b"id", idvec.as_bytes(), opt, out);
    }

    let attrs = node.attributes();

    // TODO: make optional
    // sort attributes
    let mut ids: Vec<AttributeId> = attrs.iter().map(|a| a.id).collect();
    ids.sort_by_key(|x| *x as usize);

    for aid in ids {
        let attr = attrs.get(aid).unwrap();

        if !opt.write_hidden_attributes && !attr.visible {
            continue;
        }

        out.push(b' ');
        attr.write_buf_opt(opt, out);
    }

    let ext_hash = node.unknown_attributes();
    for (name, value) in ext_hash.iter() {
        out.push(b' ');
        write_attribute(name.as_bytes(), value.as_bytes(), opt, out);
    }
}

fn write_element_start(node: &Node, opt: &WriteOptions, out: &mut Vec<u8>) {
    out.push(b'<');

    write_tag_name(&node.tag_name().unwrap(), out);
    write_attributes(node, opt, out);

    if node.has_children() {
        out.push(b'>');
    }
}

fn write_element_end(node: &Node, out: &mut Vec<u8>) {
    if node.has_children() {
        out.extend_from_slice(b"</");
        write_tag_name(&node.tag_name().unwrap(), out);
        out.push(b'>');
    } else {
        out.extend_from_slice(b"/>");
    }
}

fn process_text(iter: &mut Traverse, opt: &WriteOptions, root: &Node, depth: &Depth,
                out: &mut Vec<u8>) {
    let is_root_preserve = root.has_attribute_with_value(AttributeId::XmlSpace, "preserve");

    if !is_root_preserve {
        write_newline(opt.indent, out);
        depth.write_indent_with_step(1, out);
    }

    // Check that 'text' element contains only one text node.
    // We use 2, since 'descendants_all' includes current node.
    let is_simple_text = root.descendant_nodes().count() == 2;

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
