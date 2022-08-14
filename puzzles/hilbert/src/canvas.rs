use crate::arith::{interpolate, Bounds, Point};
use image::{ImageBuffer, Rgb};

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Color = [u16; 3];

const LINE_DENSITY: f64 = 2.0;
const SEGMENT_DENSITY: f64 = 10.0;

pub struct Canvas {
    image: Image,
    pub size: Point<u32>,
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

    pub fn save(&mut self, filename: &str) {
        self.image.save(filename).unwrap();
    }

    pub fn fill(&mut self, color: Color) {
        self.draw_rect(
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

    pub fn draw_rect(&mut self, bounds: Bounds<u32>, color: Color) {
        assert!(bounds.min.x < bounds.max.x);
        assert!(bounds.min.y < bounds.max.y);
        for x in bounds.min.x..bounds.max.x {
            for y in bounds.min.y..bounds.max.y {
                self.image.get_pixel_mut(x, y).0 = color;
            }
        }
    }

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
                self.image.get_pixel_mut(x, y).0 = color;
            }
        }
    }

    pub fn draw_curve_segment(
        &mut self,
        curve: impl Fn(f64) -> Point<f64>,
        width: f64,
        color: Color,
    ) {
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

    pub fn draw_line(&mut self, start: Point<f64>, end: Point<f64>, color: Color) {
        let len = (end - start).abs();
        let num_points = (LINE_DENSITY * len) as usize;
        for i in 0..num_points {
            let f = i as f64 / (num_points - 1) as f64;
            self.draw_point(interpolate(f, start, end), color);
        }
    }

    pub fn draw_point(&mut self, p: Point<f64>, color: Color) {
        if p.x < 0.0 || p.y < 0.0 || p.x >= self.size.x as f64 || p.y >= self.size.y as f64 {
            return;
        }
        self.image.get_pixel_mut(p.x as u32, p.y as u32).0 = color;
    }
}
