use byteorder::{LittleEndian, ReadBytesExt};
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

pub mod render;

#[derive(Debug, Default)]
pub struct LinesData {
    pub version: i32,
    pub pages: Vec<Page>,
}

#[derive(Debug)]
pub struct LinesDataReader {
    file: File,
    version: i32,
}

#[derive(Debug, Default)]
struct VersionError {
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
        write!(f, "{}", self.version_string)
    }
}

impl error::Error for VersionError {}

/// Parses data from an .rm file to `LinesData`.
/// Possible errors are `io::Error` and `VersionError`,
/// Currently, only .rm files of version 3 and 5 are supported.
impl LinesDataReader {
    pub fn read(mut file: File) -> Result<LinesData, Box<dyn error::Error>> {
        let mut buffer = [0; 33];
        file.read_exact(&mut buffer)?;
        let untrimmed_string = String::from_utf8_lossy(&buffer);
        let version_string = untrimmed_string.trim_end();
        let version = match version_string {
            "reMarkable lines with selections and layers" => {
                return Err(VersionError::boxed(version_string))
            }
            "reMarkable .lines file, version=3" => 3,
            "reMarkable .lines file, version=5" => 5,
            _ => return Err(VersionError::boxed(version_string)),
        };

        if version >= 3 {
            // Newer files have 10 more bytes in the ASCII header that we skip
            file.seek(SeekFrom::Current(10))?;
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

    fn read_number_i32(&mut self) -> Result<i32, io::Error> {
        self.file.read_i32::<LittleEndian>()
    }

    fn read_number_f32(&mut self) -> Result<f32, io::Error> {
        self.file.read_f32::<LittleEndian>()
    }

    fn read_pages(&mut self) -> Result<Vec<Page>, io::Error> {
        // From version 3(?) on, only a single page is stored per file.
        // The number of pages is not stored in the lines file any more.
        let num_pages = if self.version >= 3 {
            1
        } else {
            self.read_number_i32()?
        };
        (0..num_pages)
            .map(|_p| {
                println!("p: {} / {}", _p, num_pages);
                Ok(Page {
                    layers: self.read_layers()?,
                })
            })
            .collect()
    }

    fn read_layers(&mut self) -> Result<Vec<Layer>, io::Error> {
        let num_layers = self.read_number_i32()?;
        (0..num_layers)
            .map(|_l| {
                println!("l: {} / {}", _l, num_layers);
                Ok(Layer {
                    lines: self.read_lines()?,
                })
            })
            .collect()
    }

    fn read_lines(&mut self) -> Result<Vec<Line>, io::Error> {
        let num_lines = self.read_number_i32()?;
        (0..num_lines)
            .map(|_li| {
                println!("li: {} / {}", _li, num_lines);
                self.read_line()
            })
            .collect()
    }

    fn read_line(&mut self) -> Result<Line, io::Error> {
        Ok(Line {
            brush_type: self.read_number_i32()?,
            color: self.read_number_i32()?,
            unknown_line_attribute_1: self.read_number_i32()?,
            brush_base_size: self.read_number_f32()?,
            unkonwn_line_attribute_2: if self.version >= 5 {
                self.read_number_i32()?
            } else {
                0
            },
            points: self.read_points()?,
        })
    }

    fn read_points(&mut self) -> Result<Vec<Point>, io::Error> {
        let num_points = self.read_number_i32()?;
        (0..num_points)
            .map(|_pt| {
                println!("pt: {} / {}", _pt, num_points);
                self.read_point()
            })
            .collect()
    }

    fn read_point(&mut self) -> Result<Point, io::Error> {
        Ok(Point {
            x: self.read_number_f32()?,
            y: self.read_number_f32()?,
            speed: self.read_number_f32()?,
            direction: self.read_number_f32()?,
            width: self.read_number_f32()?,
            pressure: self.read_number_f32()?,
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

#[derive(Default, Debug)]
pub struct Line {
    pub brush_type: i32,
    pub color: i32,
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
