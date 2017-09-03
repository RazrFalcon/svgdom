#[macro_use]
extern crate bencher;
extern crate svgdom;

use std::fs;
use std::env;
use std::io::Read;

use bencher::Bencher;

use svgdom::{Document, WriteBuffer};

const TEN_MIB: usize = 10 * 1024 * 1024;

fn load_file(path: &str) -> String {
    let path = env::current_dir().unwrap().join(path);
    let mut file = fs::File::open(&path).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

macro_rules! do_parse {
    ($name:ident, $path:expr) => (
        fn $name(bencher: &mut Bencher) {
            let text = load_file($path);
            bencher.iter(|| {
                let _ = Document::from_str(&text).unwrap();
            })
        }
    )
}

do_parse!(parse_small, "benches/small.svg");
do_parse!(parse_medium, "benches/medium.svg");
do_parse!(parse_large, "benches/large.svg");

macro_rules! do_write {
    ($name:ident, $path:expr) => (
        fn $name(bencher: &mut Bencher) {
            let text = load_file($path);
            let doc = Document::from_str(&text).unwrap();
            let mut ouput_data = Vec::with_capacity(TEN_MIB);
            bencher.iter(|| {
                doc.write_buf(&mut ouput_data);
                ouput_data.clear();
            })
        }
    )
}

do_write!(write_small, "benches/small.svg");
do_write!(write_medium, "benches/medium.svg");
do_write!(write_large, "benches/large.svg");

benchmark_group!(benches1, parse_small, parse_medium, parse_large);
benchmark_group!(benches2, write_small, write_medium, write_large);
benchmark_main!(benches1, benches2);
