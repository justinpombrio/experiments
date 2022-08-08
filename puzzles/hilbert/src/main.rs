// TODO: temporary
#![allow(unused)]

mod curve;
//mod hilbert_curve;
mod oklab;
mod canvas;
mod point;

use oklab::{ColorScale};
use image::{ImageBuffer, Rgb};
use curve::{HILBERT_CURVE, LindenmayerSystem};
use canvas::{Canvas, Bounds};

const IMAGE_SIZE: u32 = 4000;
const DEPTH: usize = 5;
const CURVE_WIDTH: f64 = 1.0;

const HILBERT_DRAWING: Drawing = Drawing {
    curve: HILBERT_CURVE,
    depth: DEPTH,
    curve_width: CURVE_WIDTH * IMAGE_SIZE as f64 / 2_u32.pow(DEPTH as u32) as f64 * 0.5,
    bounds: Bounds {
        min_x: -0.5,
        min_y: -0.5,
        max_x: 2_u32.pow(DEPTH as u32) as f64 - 0.5,
        max_y: 2_u32.pow(DEPTH as u32) as f64 - 0.5,
    },
    color_scale: ColorScale {
        max_saturation: 0.127,
        min_lightness: 0.25,
        max_lightness: 0.75,
        hsv: color_scale_2,
    }
};

fn color_scale_2(f: f64) -> (f64, f64, f64) {
    let hue = cycle(f, 0.0, 1.0);
    let sat = linear_cycle(f, 0.5, 3.5, 0.5);
    let val = linear_cycle(f, 0.5, 2.0, 0.0);
    (hue, 1.0, val)
}

/// As `f` scales from 0.0 to 1.0, the result scales from `start` to `end`.
fn cycle(f: f64, start: f64, end: f64) -> f64 {
    (start + f * (end - start)) % 1.0
}

fn linear_cycle(f: f64, start: f64, end: f64, min: f64) -> f64 {
    min + (1.0 - (2.0 * cycle(f, start, end) - 1.0).abs()) * (1.0 - min)
}

struct Drawing {
    curve: LindenmayerSystem,
    depth: usize,
    curve_width: f64,
    color_scale: ColorScale,
    bounds: Bounds<f64>,
}

impl Drawing {
    fn draw_on_canvas(&self, canvas: &mut Canvas) {
        let drawing_width = self.bounds.max_x - self.bounds.min_x;
        let drawing_height = self.bounds.max_y - self.bounds.min_y;
        let points = self.curve.expand(self.depth);
        let curve_len = points.len() - 1;

        let (width, height) = (canvas.width, canvas.height);
        let mut points = points.map(move |(x, y)| {
            ((x - self.bounds.min_x) / drawing_width * width as f64,
             (y - self.bounds.min_y) / drawing_height * height as f64)
        });

        let mut start = points.next().unwrap();
        let mut middle = points.next().unwrap();
        canvas.draw_curve_segment(
            |f| (interpolate(f, (3.0 * start.0 - middle.0) / 2.0, (start.0 + middle.0) / 2.0),
                 interpolate(f, (3.0 * start.1 - middle.1) / 2.0, (start.1 + middle.1) / 2.0)),
            self.curve_width,
            self.color_scale.sample(0.0)
        );
        for (i, end) in points.enumerate() {
            let color = self.color_scale.sample((i + 1) as f64 / curve_len as f64);
            canvas.draw_curve_segment(
                |f| (
                    2.0 * f * (1.0 - f) * middle.0
                    + f * f * (start.0 + middle.0) / 2.0
                    + (1.0 - f) * (1.0 - f) * (end.0 + middle.0) / 2.0,
                    2.0 * f * (1.0 - f) * middle.1
                    + f * f * (start.1 + middle.1) / 2.0
                    + (1.0 - f) * (1.0 - f) * (end.1 + middle.1) / 2.0
                ),
                self.curve_width,
                color,
            );
            if i == curve_len - 2 {
                canvas.draw_curve_segment(
                    |f| (interpolate(f, (middle.0 + end.0)/2.0, (3.0 * end.0 - middle.0)/2.0),
                         interpolate(f, (middle.1 + end.1)/2.0, (3.0 * end.1 - middle.1)/2.0)),
                    self.curve_width,
                    self.color_scale.sample(1.0)
                );
            }
            start = middle;
            middle = end;
        }
    }
}

fn main(){
    let mut canvas = Canvas::new(IMAGE_SIZE, IMAGE_SIZE);
    canvas.fill([u16::MAX, u16::MAX, u16::MAX]);
    HILBERT_DRAWING.draw_on_canvas(&mut canvas);
    canvas.save("curve.png");
}

fn interpolate(f: f64, start: f64, end: f64) -> f64 {
    start + f * (end - start)
}
