use anyhow::{Context, Result};
use clap::{App, Arg};
use lines_are_rusty::{LayerColors, LinesData};
use std::fs::{metadata, File};
use std::io::Read;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::exit;

fn main() -> Result<()> {
    let matches = App::new("lines-are-rusty")
        .version("0.1")
        .about("Converts lines files from .rm to SVG.")
        .author("Axel Huebl <axel.huebl@plasma.ninja>")
        .arg(
            Arg::with_name("file")
                .help("The .rm (or .lines) file to read from. If omitted, data is expected to be piped in.")
                .index(1)
                .empty_values(true)
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("The file to save the rendered output to. If omitted, output is written to stdout. Required for PDF.")
        )
        .arg(
            Arg::with_name("no-auto-crop")
                .short("n")
                .long("no-crop")
                .help("Don't crop the page to fit the content")
        )
        .arg(
            Arg::with_name("custom-colors")
                .short("c")
                .long("colors")
                .help("Which colors to use for the layers. Format: L1-black,L1-gray,L1-white;...;L5-black,L5-gray,L5-white")
                .default_value("black,gray,white;black,gray,white;black,gray,white;black,gray,white;black,gray,white")
        )
        .arg(
            Arg::with_name("output-type")
                .short("t")
                .long("to")
                .takes_value(true)
                .help("Output type. If present, overrides the type determined by the output file extension. Defaults to svg.")
                .possible_values(&["svg", "pdf", "xfdf"])
        )
        .arg(
            Arg::with_name("debug-dump")
            .short("d")
            .long("debug-dump")
            .help("When rendering SVG, write debug information about lines and points into the SVG as tooltips")
        )
        .get_matches();
    let output_filename = matches.value_of("output");
    let output_type_string = matches.value_of("output-type").or({
        output_filename
            .and_then(|output_filename| Path::new(output_filename).extension())
            .and_then(|extension| extension.to_str())
    });
    let output_type = match output_type_string {
        Some(output_type_string) => match output_type_string.to_lowercase().as_ref() {
            "svg" => OutputType::SVG,
            "pdf" => OutputType::PDF,
            "xfdf" => OutputType::XFDF,
            _ => {
                eprintln!("Unsupported output file extension {}", output_type_string);
                exit(1);
            }
        },
        None => OutputType::SVG,
    };


    let auto_crop = !matches.is_present("no-auto-crop");
    let colors = matches
        .value_of("custom-colors")
        .unwrap_or_else(|| unreachable!());

    let layer_colors = lines_are_rusty::LayerColors {
        colors: colors
            .split(";")
            .map(|layer| {
                let c = layer.split(",").collect::<Vec<&str>>();
                if c.len() != 3 {
                    eprintln!("Expected 3 colors per layer. Found: {}", layer);
                    exit(1);
                }
                (c[0].to_string(), c[1].to_string(), c[2].to_string())
            })
            .collect(),
    };

    let debug_dump = matches.is_present("debug-dump");
    if debug_dump && (output_type != OutputType::SVG) {
        eprintln!("Warning: debug-dump only has an effect when writing SVG output");
    }

    let options = Options {
        output_type,
        output_filename,
        layer_colors,
        auto_crop,
        debug_dump,
    };

    let mut output = BufWriter::new(match output_filename {
        Some(output_filename) => Box::new(
            File::create(output_filename)
                .context(format!("Can't create {}", output_filename))?,
        ),
        None => Box::new(io::stdout()) as Box<dyn Write>,
    });

    match matches.value_of("file") {
        None => process_single_file(&mut io::stdin(), &mut output, options)?,
        Some(filename) => {
            let metadata = metadata(filename).context(format!("Can't access input file {}", filename))?;
            if metadata.is_dir() {
                process_notebook(filename, &mut output, options);
            } else {
                let mut input = File::open(filename).context(format!("Can't open input file {}", filename))?;
                process_single_file(&mut input, &mut output, options)?;
            }
        },
    };

    eprintln!("done.");

    Ok(())
}

fn process_single_file(mut input: &mut dyn Read, mut output: &mut dyn Write, opts: Options) -> Result<()> {
    let lines_data = LinesData::parse(&mut input).context("Failed to parse lines data")?;

    Ok(match opts.output_type {
        OutputType::SVG => {
            lines_are_rusty::render_svg(output, &lines_data.pages[0], opts.auto_crop, &opts.layer_colors, opts.debug_dump)
        }
        OutputType::PDF => {
            // Alas, the pdf-canvas crate insists on writing to a File instead of a Write
            let pdf_filename = opts
                .output_filename
                .context("Output file needed for PDF output")?;
            lines_are_rusty::render_pdf(pdf_filename, &lines_data.pages);
        }
        OutputType::XFDF => {
            lines_are_rusty::render_xfdf(&mut output, &lines_data.pages[0], &opts.layer_colors)
        }
    })
}

fn process_notebook(filename: &str, mut output: &mut dyn Write, opts: Options) {

}

#[derive(Debug, PartialEq)]
enum OutputType {
    SVG,
    PDF,
    XFDF,
}

struct Options<'a> {
    output_type: OutputType,
    output_filename: Option<&'a str>,
    layer_colors: LayerColors,
    auto_crop: bool,
    debug_dump: bool,
}

trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self, message: &str) -> T;
}

impl<T, E: std::fmt::Display> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self, message: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}\n{}", message, e);
                exit(1);
            }
        }
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_exit(self, message: &str) -> T {
        self.unwrap_or_else(|| {
            eprintln!("{}", message);
            exit(1)
        })
    }
}
