use crate::arith::{interpolate, Bounds, Point};
use crate::srgb::Color;
use image::{ImageBuffer, Rgb};

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;

const LINE_DENSITY: f64 = 2.0;
const SEGMENT_DENSITY: f64 = 10.0;
const CIRCLE_DENSITY: f64 = 4.0;

/// Draw to a PNG
pub struct Canvas {
    image: Image,
    size: Point<u32>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            image: ImageBuffer::<Rgb<u16>, _>::new(width, height),
            size: Point {
                x: width,
                y: height,
            },
        }
    }

    /// Save this image as a PNG file.
    pub fn save(&mut self, filename: &str) {
        self.image.save(filename).unwrap();
    }

    /// Fill the entire canvas with a background color.
    pub fn fill(&mut self, color: Color) {
        self.fill_rect(
            Bounds {
                min: Point { x: 0, y: 0 },
                max: Point {
                    x: self.image.width(),
                    y: self.image.height(),
                },
            },
            color,
        );
    }

    /// Fill a rectangle with the given color.
    pub fn fill_rect(&mut self, bounds: Bounds<u32>, color: Color) {
        assert!(bounds.min.x < bounds.max.x);
        assert!(bounds.min.y < bounds.max.y);
        for x in bounds.min.x..bounds.max.x {
            for y in bounds.min.y..bounds.max.y {
                self.image.get_pixel_mut(x, y).0 = color.0;
            }
        }
    }

    /// Paint a rectangle with the given color function.
    pub fn paint_rect(&mut self, bounds: Bounds<u32>, paint: impl Fn(Point<u32>) -> Color) {
        assert!(bounds.min.x < bounds.max.x);
        assert!(bounds.min.y < bounds.max.y);
        for x in bounds.min.x..bounds.max.x {
            for y in bounds.min.y..bounds.max.y {
                let point = Point { x, y };
                self.image.get_pixel_mut(x, y).0 = paint(point).0;
            }
        }
    }

    /// Within the rectangle `image_bounds`, draw a `num_checkers.x` by `num_checkers.y`
    /// checkerboard using the colors `color_1` and `color_2`.
    pub fn draw_checkerboard(
        &mut self,
        image_bounds: Bounds<u32>,
        num_checkers: Point<u32>,
        color_1: Color,
        color_2: Color,
    ) {
        let size = image_bounds.max - image_bounds.min;
        for x in image_bounds.min.x..image_bounds.max.x {
            for y in image_bounds.min.y..image_bounds.max.y {
                let point = Point { x, y };
                let parity = (point - image_bounds.min) * num_checkers / size;
                let color = if (parity.x + parity.y) % 2 == 0 {
                    color_1
                } else {
                    color_2
                };
                self.image.get_pixel_mut(x, y).0 = color.0;
            }
        }
    }

    /// Draw a curve with the given width and color. The curve itself is given by the parametric
    /// function `curve(f)` for `0.0 <= f < = 1`.
    pub fn draw_curve(&mut self, curve: impl Fn(f64) -> Point<f64>, width: f64, color: Color) {
        let start = curve(0.0);
        let end = curve(1.0);
        let len = (start - end).abs();
        let num_points = (SEGMENT_DENSITY * len) as usize;
        for i in 0..num_points {
            // compute first and second point
            let f0 = i as f64 / (num_points - 1) as f64;
            let f1 = (i + 1) as f64 / (num_points - 1) as f64;
            let p0 = curve(f0);
            let p1 = curve(f1);

            // compute normalized orthogonal vector
            let d = p1 - p0;
            let ortho = d.rotate_quarter_turn() / d.abs();

            // draw line from left point to right point
            let left = p0 + ortho * width;
            let right = p0 - ortho * width;
            self.draw_line(left, right, color);
        }
    }

    pub fn draw_circle(&mut self, point: Point<f64>, radius: f64, color: Color) {
        // What is a circle but a bunch of lines
        let num_lines = (CIRCLE_DENSITY * radius) as usize;
        for i in 0..num_lines {
            let angle = i as f64 / num_lines as f64 / 2.0;
            let delta = Point::cis(angle) * radius;
            self.draw_line(point - delta, point + delta, color);
        }
    }

    /// Fill the canvas with `fill_color` starting from the center, so long as the pixels have
    /// `background` color.
    pub fn bucket_fill(&mut self, fill_color: Color, background: Color) {
        let mut frontier = vec![self.size / 2];
        while let Some(point) = frontier.pop() {
            if self.image.get_pixel(point.x, point.y).0 != background.0 {
                continue;
            }
            self.image.get_pixel_mut(point.x, point.y).0 = fill_color.0;
            if point.x > 0 {
                frontier.push(Point {
                    x: point.x - 1,
                    y: point.y,
                });
            }
            if point.y > 0 {
                frontier.push(Point {
                    x: point.x,
                    y: point.y - 1,
                });
            }
            if point.x < self.size.x - 1 {
                frontier.push(Point {
                    x: point.x + 1,
                    y: point.y,
                });
            }
            if point.y < self.size.y - 1 {
                frontier.push(Point {
                    x: point.x,
                    y: point.y + 1,
                });
            }
        }
    }

    /// Draw a line of the given color, of width 1 pixel.
    pub fn draw_line(&mut self, start: Point<f64>, end: Point<f64>, color: Color) {
        let len = (end - start).abs();
        let num_points = (LINE_DENSITY * len) as usize;
        for i in 0..num_points {
            let f = i as f64 / (num_points - 1) as f64;
            self.draw_point(interpolate(f, start, end), color);
        }
    }

    /// Set the color of a single pixel.
    pub fn draw_point(&mut self, p: Point<f64>, color: Color) {
        if p.x < 0.0 || p.y < 0.0 || p.x >= self.size.x as f64 || p.y >= self.size.y as f64 {
            return;
        }
        self.image.get_pixel_mut(p.x as u32, p.y as u32).0 = color.0;
    }
}
