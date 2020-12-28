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

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => {println!("{}", e); return}
    };
    let lines_data = match LinesDataReader::read(file) {
        Ok(lines_data) => lines_data,
        Err(e) => {println!("{}", e); return}
    };

    println!("\ndone. read {} pages.", lines_data.pages.len());

    render(&output_filename, &lines_data.pages);
}
