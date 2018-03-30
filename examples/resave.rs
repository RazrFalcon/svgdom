extern crate svgdom;
extern crate time;

use std::env;
use std::io::{Read,Write};
use std::fs::File;

use svgdom::{Document, WriteBuffer};

fn main() {
    let start = time::precise_time_ns();

    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        println!("Usage:\n\tresave in.svg out.svg");
        return;
    }

    let mut file = File::open(&args[1]).unwrap();
    let length = file.metadata().unwrap().len() as usize;

    let mut input_data = String::with_capacity(length + 1);
    file.read_to_string(&mut input_data).unwrap();

    let doc = Document::from_str(&input_data).unwrap();

    let mut output_data = Vec::new();
    doc.write_buf(&mut output_data);

    let mut f = File::create(&args[2]).unwrap();
    f.write_all(&output_data).unwrap();

    let end = time::precise_time_ns();
    println!("Elapsed: {:.4}ms", (end - start) as f64 / 1_000_000.0);
}
