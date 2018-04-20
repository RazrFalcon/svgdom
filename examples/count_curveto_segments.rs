extern crate svgdom;

use std::env;
use std::io::Read;
use std::fs::File;

use svgdom::{ AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathCommand };

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("Usage:\n\tcount_curveto_segments in.svg");
        return;
    }

    let mut file = File::open(&args[1]).unwrap();
    let length = file.metadata().unwrap().len() as usize;

    let mut input_data = String::with_capacity(length + 1);
    file.read_to_string(&mut input_data).unwrap();

    let doc = Document::from_str(&input_data).unwrap();

    let mut count = 0;

    for (id, node) in doc.root().descendants().svg() {
        if id == ElementId::Path {
            let attrs = node.attributes();
            if let Some(&AttributeValue::Path(ref path)) = attrs.get_value(AttributeId::D) {
                count += path.iter().filter(|seg| seg.cmd() == PathCommand::CurveTo).count();
            }
        }
    }

    println!("This file contains {} CurveTo segments.", count);
}
