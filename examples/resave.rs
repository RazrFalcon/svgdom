use std::env;
use std::fs;

use svgdom::WriteBuffer;

fn main() -> Result<(), Box<std::error::Error>> {
    fern::Dispatch::new()
        .format(|out, message, record|
            out.finish(format_args!("{}: {}", record.level(), message))
        ).chain(std::io::stderr()).apply()?;

    let start = time::precise_time_ns();

    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tresave in.svg out.svg");
        std::process::exit(1);
    }

    let input_data = fs::read_to_string(&args[1])?;
    let doc = svgdom::Document::from_str(&input_data)?;

    let mut output_data = Vec::new();
    doc.write_buf(&mut output_data);

    fs::write(&args[2], &output_data)?;

    let end = time::precise_time_ns();
    println!("Elapsed: {:.4}ms", (end - start) as f64 / 1_000_000.0);

    Ok(())
}
