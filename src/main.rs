use clap::{App, Arg};
use std::fs::File;
use lines_are_rusty::*;
use lines_are_rusty::render::*;

fn main() {
    let matches = App::new("lines-are-rusty")
        .version("0.1")
        .about("Converts lines files from .rm to SVG.")
        .author("Axel Huebl <axel.huebl@plasma.ninja>")
        .arg(
            Arg::with_name("file")
                .help("The file to read from")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("The file to save the PDF to")
                .required(true)
                .index(2),
        )
        .get_matches();
    let filename = matches
        .value_of("file")
        .expect("Expected required filename.");
    let output_filename = matches
        .value_of("output")
        .expect("Expected required filename.");

    // Load the file into a Vec<u8>
    let pages = LinesFile::new(File::open(filename).unwrap()).read_pages();

    println!("\ndone. read {} pages.", pages.len());

    render(&output_filename, &pages);
}
