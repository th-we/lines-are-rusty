use byteorder::{LittleEndian, ReadBytesExt};
use std::io;
use crate::{Page, Layer, Line, Point};

pub(crate) struct LinesDataReader<'a> {
    pub file: &'a mut dyn io::Read,
    pub version: i32,
}

impl LinesDataReader<'_> {
    fn read_number_i32(&mut self) -> Result<i32, io::Error> {
        self.file.read_i32::<LittleEndian>()
    }

    fn read_number_f32(&mut self) -> Result<f32, io::Error> {
        self.file.read_f32::<LittleEndian>()
    }

    pub fn read_pages(&mut self) -> Result<Vec<Page>, io::Error> {
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
