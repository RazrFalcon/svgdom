// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate svgdom;

use std::env;
use std::io::{Read, Write};
use std::fs::File;

use svgdom::{
    Attribute,
    AttributeId,
    Document,
    Node,
    ElementId,
    NodeEdge,
    NodeType,
    WriteBuffer,
    WriteOptions,
};
use svgdom::writer::Depth;
use svgdom::writer;

fn main() {
    // This example is showing how to write a document with custom formatting.
    //
    // For this example, we will write each attribute on a new line.

    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        println!("Usage:\n\tcustom_writer in.svg out.svg");
        return;
    }

    // load file
    let mut file = File::open(&args[1]).unwrap();
    let length = file.metadata().unwrap().len() as usize;

    let mut input_data = Vec::with_capacity(length + 1);
    file.read_to_end(&mut input_data).unwrap();

    let doc = Document::from_data(&input_data).unwrap();

    // write file
    let mut opt = WriteOptions::default();
    opt.indent = 2;

    let mut out: Vec<u8> = Vec::with_capacity(length + 1);
    write_dom(&doc, &opt, &mut out);

    let mut f = File::create(&args[2]).unwrap();
    f.write_all(&out).unwrap();
}

// Firstly, we will copy the default `write_dom` implementation from the `writer` module.
fn write_dom(doc: &Document, opt: &WriteOptions, out: &mut Vec<u8>) {
    let mut depth = Depth::new(opt.indent);
    let mut iter = doc.root().traverse();

    while let Some(edge) = iter.next() {
        match edge {
            NodeEdge::Start(node) => {
                match node.node_type() {
                    NodeType::Element => {
                        // it's mostly a direct copy of the `writer::write_start_edge` method
                        depth.write_indent(out);

                        // here we invoking own `write_element_start` implementation
                        write_element_start(&node, &depth, opt, out);

                        // below is the same code as in the `writer::write_start_edge` method
                        if node.is_tag_name(ElementId::Text) && node.has_children() {
                            writer::process_text(&mut iter, opt, &node, &depth, out);
                            writer::write_newline(opt.indent, out);
                            return;
                        }

                        if node.has_children() {
                            depth.value += 1;
                            writer::write_newline(opt.indent, out);
                        }
                    }
                    // we don't care about other node types, so we will use a default implementation
                    _ => writer::write_start_edge(&node, &mut iter, &mut depth, opt, out),
                }
            }
            NodeEdge::End(node) => {
                // we don't care about closing tags, so we will use a default implementation
                writer::write_end_edge(&node, &mut depth, opt.indent, out)
            }
        }
    }
}

fn write_element_start(node: &Node, depth: &Depth, opt: &WriteOptions, out: &mut Vec<u8>) {
    out.push(b'<');
    writer::write_tag_name(&node.tag_name().unwrap(), out);

    // write `id`
    // in libsvgdom the `id` attribute is not a part of attributes list,
    // so we write it separately
    if node.has_id() {
        let attr = Attribute::new(AttributeId::Id, node.id().clone());
        write_attribute(&attr, &depth, opt, out);
    }

    // write attributes
    let attrs = node.attributes();
    for attr in attrs.iter() {
        write_attribute(attr, &depth, opt, out);
    }

    if node.has_children() {
        out.push(b'>');
    }
}

// writes each attribute on new line
fn write_attribute(attr: &Attribute, depth: &Depth, opt: &WriteOptions, out: &mut Vec<u8>) {
    out.push(b'\n');
    depth.write_indent_with_step(1, out);
    out.push(b' ');
    attr.write_buf_opt(&opt, out);
}
