// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate svgdom;

use svgdom::{Document, AttributeId, AttributeValue};

use std::env;
use std::io::Read;
use std::fs::File;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcount_curveto_segments in.svg");
        return;
    }

    let mut file = File::open(&args[1]).unwrap();
    let length = file.metadata().unwrap().len() as usize;

    let mut input_data = Vec::with_capacity(length + 1);
    file.read_to_end(&mut input_data).unwrap();

    let doc = Document::from_data(&input_data).unwrap();

    let mut count = 0;

    for node in doc.descendants() {
        let attrs = node.attributes();
        match attrs.get(AttributeId::D) {
            Some(attr) => {
                match attr.value {
                    AttributeValue::Path(ref path) => {
                        count += path.d.iter().filter(|seg| seg.cmd().is_curve_to()).count();
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }

    println!("This file contains {} CurveTo segments.", count);
}
