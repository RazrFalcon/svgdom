// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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

    let mut input_data = Vec::with_capacity(length + 1);
    file.read_to_end(&mut input_data).unwrap();

    let doc = Document::from_data(&input_data).unwrap();

    let mut ouput_data = Vec::new();
    doc.write_buf(&mut ouput_data);

    let mut f = File::create(&args[2]).unwrap();
    f.write_all(&ouput_data).unwrap();

    let end = time::precise_time_ns();
    println!("Elapsed: {:.4}ms", (end - start) as f64 / 1_000_000.0);
}
