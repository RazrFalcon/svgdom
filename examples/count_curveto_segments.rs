use std::env;
use std::fs;

use svgdom::{AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathCommand};

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\tcount_curveto_segments in.svg");
        std::process::exit(1);
    }

    let input_data = fs::read_to_string(&args[1])?;
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

    Ok(())
}
