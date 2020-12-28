use crate::lines_data_reader::LinesDataReader;
use std::error;
use std::fmt;
use std::io;

mod lines_data_reader;

pub mod render;
pub use render::{render_pdf, render_svg};

#[derive(Debug, Default)]
pub struct VersionError {
    version_string: String,
}

impl VersionError {
    fn boxed(version_string: &str) -> Box<VersionError> {
        Box::new(VersionError {
            version_string: version_string.to_string(),
        })
    }
}

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported version string: {}", self.version_string)
    }
}

impl error::Error for VersionError {}

#[derive(Debug, Default)]
pub struct LinesData {
    pub version: i32,
    pub pages: Vec<Page>,
}

impl LinesData {
    /// Parses data from an .rm file to `LinesData`.
    /// Possible errors are `io::Error` and `VersionError`,
    /// Currently, only .rm files of version 3 and 5 are supported.
    pub fn read(file: &mut dyn io::Read) -> Result<LinesData, Box<dyn error::Error>> {
        let mut buffer = [0; 33];
        file.read_exact(&mut buffer)?;
        let untrimmed_string = String::from_utf8_lossy(&buffer);
        let version_string = untrimmed_string.trim_end();
        let version = match version_string {
            "reMarkable lines with selections and layers" => {
                // early version of the format that is not supported
                return Err(VersionError::boxed(version_string));
            }
            "reMarkable .lines file, version=3" => 3,
            "reMarkable .lines file, version=5" => 5,
            _ => return Err(VersionError::boxed(version_string)),
        };

        if version >= 3 {
            // Newer files have 10 more bytes in the ASCII header that we skip
            file.read_exact(&mut [0; 10])?;
        }

        let mut reader = LinesDataReader {
            file: file,
            version: version,
        };

        Ok(LinesData {
            version: version,
            pages: reader.read_pages()?,
        })
    }
}

#[derive(Default, Debug)]
pub struct Page {
    pub layers: Vec<Layer>,
}

#[derive(Default, Debug)]
pub struct Layer {
    // TODO: Add layer names
    pub lines: Vec<Line>,
}

#[derive(Debug)]
pub enum BrushType {
    BallPoint,
    Marker,
    Fineliner,
    SharpPencil,
    TiltPencil,
    Brush,
    Highlighter,
    Eraser,
    EraseArea,
    Calligraphy,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black,
    Grey,
    White,
}

#[derive(Debug)]
pub struct Line {
    pub brush_type: BrushType,
    pub color: Color,
    pub unknown_line_attribute_1: i32,
    pub brush_base_size: f32,
    pub unkonwn_line_attribute_2: i32,
    pub points: Vec<Point>,
}

#[derive(Default, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub direction: f32,
    pub width: f32,
    pub pressure: f32,
}

pub struct LayerColors {
    pub colors: Vec<(String, String, String)>,
}
