use crate::{BrushType, Color, Layer, Line, Page, Point};
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::error;
use std::io;

pub(crate) struct LinesDataReader<'a> {
    pub file: &'a mut dyn io::Read,
    pub version: i32,
}

impl LinesDataReader<'_> {
    fn read_i32(&mut self) -> Result<i32, io::Error> {
        self.file.read_i32::<LittleEndian>()
    }

    fn read_f32(&mut self) -> Result<f32, io::Error> {
        self.file.read_f32::<LittleEndian>()
    }

    pub fn read_pages(&mut self) -> Result<Vec<Page>, Box<dyn error::Error>> {
        // From version 3(?) on, only a single page is stored per file.
        // The number of pages is not stored in the lines file any more.
        let num_pages = if self.version >= 3 {
            1
        } else {
            self.read_i32()?
        };
        (0..num_pages)
            .map(|_| {
                Ok(Page {
                    layers: self.read_layers()?,
                })
            })
            .collect()
    }

    fn read_layers(&mut self) -> Result<Vec<Layer>, Box<dyn error::Error>> {
        let num_layers = self.read_i32()?;
        (0..num_layers)
            .map(|_| {
                Ok(Layer {
                    lines: self.read_lines()?,
                })
            })
            .collect()
    }

    fn read_lines(&mut self) -> Result<Vec<Line>, Box<dyn error::Error>> {
        let num_lines = self.read_i32()?;
        (0..num_lines).map(|_| self.read_line()).collect()
    }

    fn read_line(&mut self) -> Result<Line, Box<dyn error::Error>> {
        Ok(Line {
            brush_type: BrushType::try_from(self.read_i32()?)?,
            color: Color::try_from(self.read_i32()?)?,
            unknown_line_attribute_1: self.read_i32()?,
            brush_base_size: self.read_f32()?,
            unkonwn_line_attribute_2: if self.version >= 5 {
                self.read_i32()?
            } else {
                0
            },
            points: self.read_points()?,
        })
    }

    fn read_points(&mut self) -> Result<Vec<Point>, io::Error> {
        let num_points = self.read_i32()?;
        (0..num_points).map(|_| self.read_point()).collect()
    }

    fn read_point(&mut self) -> Result<Point, io::Error> {
        Ok(Point {
            x: self.read_f32()?,
            y: self.read_f32()?,
            speed: self.read_f32()?,
            direction: self.read_f32()?,
            width: self.read_f32()?,
            pressure: self.read_f32()?,
        })
    }
}

impl std::convert::TryFrom<i32> for BrushType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            21 => Ok(BrushType::Calligraphy),
            15 => Ok(BrushType::BallPoint),
            16 => Ok(BrushType::Marker),
            17 => Ok(BrushType::Fineliner),
            13 => Ok(BrushType::SharpPencil),
            14 => Ok(BrushType::TiltPencil),
            12 => Ok(BrushType::Brush),
            18 => Ok(BrushType::Highlighter),
            6 => Ok(BrushType::Eraser),
            8 => Ok(BrushType::EraseArea),
            v => Err(format!("Unknown brush type: {}", v)),
        }
    }
}

impl TryFrom<i32> for Color {
    type Error = String;
    fn try_from(color_i: i32) -> Result<Self, Self::Error> {
        match color_i {
            0 => Ok(Color::Black),
            1 => Ok(Color::Grey),
            2 => Ok(Color::White),
            _ => Err(format!("Unknown color: {}", color_i)),
        }
    }
}
