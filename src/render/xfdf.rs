use crate::render::renderlib::{line_to_css_color, segment_quads, BoundingBox};
use crate::*;
use maud::PreEscaped;
use std::io::Write;
// Alias this because html! would be irritating for generating XML
use maud::html as xml;

// TODO: Coordinate system transformation:
// * Reverse coordinates
// * scale to the PDF. reMarkable puts the PDF in the top left corner and scales
//   it to fit in the window (before transforming)
// * transform according to the transform matrix from the .content file

fn render_highlighter_line(page_index: i32, line: &Line, color: &str) -> PreEscaped<String> {
    // Draw a trapezoid for each segment of the highlight polyline.
    // the parallel sides of the trapezoid are parallel to the polyline segment,
    // the other sides are angle bisectors in the start/endpoints. To calculate
    // those, we also have to take into account the preceding and following
    // segments of each
    xml! {
        highlight
            page=(page_index)
            // TODO: Make highlighter color configurable
            // color=(color)
            color="#ffff00"
            // TODO: Adjust rect by line width
            rect=(rect_attribute(line))
            title="reMarkable"
            subject="Highlighter"
            coords=(segment_quads(line).iter().map(|c| c.to_string()).collect::<Vec<String>>().join(","))
            {}
    }
}

fn rect_attribute(line: &Line) -> String {
    let b = BoundingBox::new().enclose_line(line);
    format!("{},{},{},{}", b.min_x, b.min_y, b.max_x, b.max_y)
}

fn render_line(page_index: i32, line: &Line, color: &str) -> PreEscaped<String> {
    xml! {
        ink
            page=(page_index)
            color=(color)
            rect=(rect_attribute(line))
            title="reMarkable"
            subject=(format!("{:#?}", line.brush_type))
            width=(line.average_width())
        {
            inklist {
                gesture {(
                    line.points.iter()
                        .map(|p| format!("{},{}", p.x, p.y))
                        .collect::<Vec<_>>()
                        .join(";")
                )}
            }
        }
    }
}

pub fn render_xfdf(output: &mut dyn Write, page: &Page, layer_colors: &LayerColors) {
    // TODO: Process all pages for PDF
    let xfdf = xml! {
        xfdf xmlns="http://ns.adobe.com/xfdf/" {
            // TODO: Put PDF path in the href
            f href="" {}
            annots {
                @let page_index = 0;
                @for (layer_id, layer) in page.layers.iter().enumerate() {
                    @for line in layer.lines.iter() {
                        @let color = line_to_css_color(&line, layer_id, layer_colors);
                        @match line.brush_type {
                            _ if line.points.is_empty() => {},
                            BrushType::Highlighter => (render_highlighter_line(page_index, line, color)),
                            _ => (render_line(page_index, line, color)),
                        }
                    }
                }
            }
        }
    };
    output.write(xfdf.into_string().as_bytes()).unwrap();
}
