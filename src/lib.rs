use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::Read;

pub mod render;

#[derive(Debug)]
pub struct LinesFile {
    pub version: i32,
    file: File,
}

impl LinesFile {
    pub fn new(mut file: File) -> LinesFile {
        let mut buffer = [0; 33];
        file.read_exact(&mut buffer);
        let version = match String::from_utf8_lossy(&buffer).trim_end() {
            "reMarkable .lines file, version=3" => 3,
            "reMarkable .lines file, version=5" => 5,
            _ => panic!(),
        };

        LinesFile {
            version: version,
            file: file,
        }
    }

    fn read_number_i32(&mut self) -> i32 {
        // TODO implement if let Some(...)
        self.file.read_i32::<LittleEndian>().unwrap()
    }
    fn read_number_f32(&mut self) -> f32 {
        // TODO implement if let Some(...)
        self.file.read_f32::<LittleEndian>().unwrap()
    }
    pub fn read_pages(&mut self) -> Vec<Page> {
        let mut pages = vec![];
        let num_pages = 1;
        println!("p: 0 / {}", num_pages);
        let new_page = Page {
            layers: self.read_layers(),
        };
        pages.push(new_page);
        pages
    }

    fn read_layers(&mut self) -> Vec<Layer> {
        let mut layers = vec![];
        let num_layers = self.read_number_i32();
        for _l in 0..num_layers {
            println!("l: {} / {}", _l, num_layers);
            let new_layer = Layer {
                lines: self.read_lines(),
            };
            layers.push(new_layer);
        }
        layers
    }

    fn read_lines(&mut self) -> Vec<Line> {
        let mut lines = vec![];
        let num_lines = self.read_number_i32();
        for _li in 0..num_lines {
            println!("li: {} / {}", _li, num_lines);
            lines.push(self.read_line());
        }
        lines
    }

    fn read_line(&mut self) -> Line {
        Line {
            brush_type: self.read_number_i32(),
            color: self.read_number_i32(),
            unknown_line_attribute_1: self.read_number_i32(),
            brush_base_size: self.read_number_f32(),
            unkonwn_line_attribute_2: if self.version >= 5 {
                self.read_number_i32()
            } else {
                0
            },
            points: self.read_points(),
        }
    }
    fn read_points(&mut self) -> Vec<Point> {
        let mut points = Vec::<Point>::default();
        let num_points = self.read_number_i32();
        for _pt in 0..num_points {
            println!("pt: {} / {}", _pt, num_points);
            points.push(self.read_point());
        }
        points
    }

    fn read_point(&mut self) -> Point {
        Point {
            x: self.read_number_f32(),
            y: self.read_number_f32(),
            speed: self.read_number_f32(),
            direction: self.read_number_f32(),
            width: self.read_number_f32(),
            pressure: self.read_number_f32(),
        }
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
