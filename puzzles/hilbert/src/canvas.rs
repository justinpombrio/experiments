use crate::interpolate;
use image::{ImageBuffer, Rgb};
use std::mem;

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Color = [u16; 3];

pub struct Canvas {
    image: Image,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds<N: Copy> {
    pub min_x: N,
    pub min_y: N,
    pub max_x: N,
    pub max_y: N,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        Canvas {
            image: ImageBuffer::<Rgb<u16>, _>::new(width, height),
            width,
            height,
        }
    }

    pub fn save(&mut self, filename: &str) {
        self.image.save(filename).unwrap();
    }

    pub fn fill(&mut self, color: Color) {
        self.draw_rect(
            Bounds {
                min_x: 0,
                min_y: 0,
                max_x: self.image.width(),
                max_y: self.image.height(),
            },
            color,
        );
    }

    pub fn draw_rect(&mut self, mut bounds: Bounds<u32>, color: Color) {
        if bounds.min_x > bounds.max_x {
            mem::swap(&mut bounds.min_x, &mut bounds.max_x);
        }
        if bounds.min_y > bounds.max_y {
            mem::swap(&mut bounds.min_y, &mut bounds.max_y);
        }
        for x in bounds.min_x..bounds.max_x {
            for y in bounds.min_y..bounds.max_y {
                self.image.get_pixel_mut(x, y).0 = color;
            }
        }
    }

    pub fn draw_checkerboard(
        &mut self,
        image_bounds: Bounds<u32>,
        num_checkers: (u32, u32),
        color_1: Color,
        color_2: Color,
    ) {
        let width = image_bounds.max_x - image_bounds.min_x;
        let height = image_bounds.max_y - image_bounds.min_y;

        for x in image_bounds.min_x..image_bounds.max_x {
            for y in image_bounds.min_y..image_bounds.max_y {
                let x_parity = (x - image_bounds.min_x) * num_checkers.0 / width;
                let y_parity = (y - image_bounds.min_y) * num_checkers.1 / height;
                let color = if (x_parity + y_parity) % 2 == 0 {
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
        curve: impl Fn(f64) -> (f64, f64),
        width: f64,
        color: Color,
    ) {
        let start = curve(0.0);
        let end = curve(1.0);
        let approx_len = (end.0 - start.0).abs() + (end.1 - start.1).abs();
        let num_points = (2.0 * approx_len) as usize;
        for i in 0..num_points {
            // compute first and second point
            let f0 = i as f64 / (num_points - 1) as f64;
            let f1 = (i + 1) as f64 / (num_points - 1) as f64;
            let (x0, y0) = curve(f0);
            let (x1, y1) = curve(f1);

            // compute normalized orthogonal vector
            let (dx, dy) = (x1 - x0, y1 - y0);
            let d_len = (dx * dx + dy * dy).sqrt();
            let (ox, oy) = (dy / d_len, -dx / d_len);

            // draw line from left point to right point
            let left = (x0 + ox * width, y0 + oy * width);
            let right = (x0 - ox * width, y0 - oy * width);
            self.draw_line(left, right, color);
        }
    }

    fn draw_orthogonal_line(
        &mut self,
        start: (f64, f64),
        end: (f64, f64),
        width: f64,
        color: Color
    ) {
        let (x0, y0) = start;
        let (x1, y1) = end;

        // normalized orthogonal vector
        let (dx, dy) = (x1 - x0, y1 - y0);
        let d_len = (dx * dx + dy * dy).sqrt();
        let (ox, oy) = (dy / d_len, -dx / d_len);

        let start = (x0 + ox * width, y0 + oy * width);
        let end = (x0 - ox * width, y0 - oy * width);
        self.draw_line(start, end, color);
    }

    pub fn draw_line(&mut self, start: (f64, f64), end: (f64, f64), color: Color) {
        let (x0, y0) = start;
        let (x1, y1) = end;
        let approx_len = (x1 - x0).abs() + (y1 - y0).abs();
        let num_points = (2.0 * approx_len) as usize;
        for i in 0..num_points {
            let f = (i as f64 / (num_points - 1) as f64);
            let x = interpolate(f, x0, x1);
            let y = interpolate(f, y0, y1);
            self.draw_point(x, y, color);
        }
    }

    pub fn draw_point(&mut self, x: f64, y: f64, color: Color) {
        if x < 0.0 || y < 0.0 || x >= self.width as f64 || y >= self.height as f64 {
            return;
        }
        self.image.get_pixel_mut(x as u32, y as u32).0 = color;
    }
}
