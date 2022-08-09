// TODO: temporary
#![allow(unused)]

mod curve;
mod oklab;
mod canvas;
mod arith;

use oklab::ColorScale;
use image::{ImageBuffer, Rgb};
use curve::{HILBERT_CURVE, LindenmayerSystem};
use canvas::Canvas;
use arith::{Point, Bounds, interpolate};

const IMAGE_SIZE: u32 = 4000;
const DEPTH: usize = 4;
const CURVE_WIDTH: f64 = 1.0;

const HILBERT_DRAWING: Drawing = Drawing {
    curve: HILBERT_CURVE,
    depth: DEPTH,
    curve_width: CURVE_WIDTH * IMAGE_SIZE as f64 / 2_u32.pow(DEPTH as u32) as f64 * 0.5,
    bounds: Bounds {
        min: Point { x: -0.5, y: -0.5 },
        max: Point {
            x: 2_u32.pow(DEPTH as u32) as f64 - 0.5,
            y: 2_u32.pow(DEPTH as u32) as f64 - 0.5,
        }
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

/// As `f` scales from 0.0 to 1.0, the result varies between `c(start)` and `c(end)`, 
/// where `c` is a cyclic linear function varying between 0.0 at `0, 1, 2, ...` and 1.0 at
/// `0.5, 1.5, 2.5, ...`.
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
        let drawing_size = self.bounds.max - self.bounds.min;
        let points = self.curve.expand(self.depth);
        let curve_len = points.len() - 1;

        let canvas_size = Point {
            x: canvas.size.x as f64,
            y: canvas.size.y as f64,
        };
        let mut points = points.map(move |point| {
            (point - self.bounds.min) / drawing_size * canvas_size
        });

        let mut start = points.next().unwrap();
        let mut middle = points.next().unwrap();
        // first segment
        canvas.draw_curve_segment(
            |f| interpolate(f, (start * 3.0 - middle) / 2.0, (start + middle) / 2.0),
            self.curve_width,
            self.color_scale.sample(0.0)
        );
        for (i, end) in points.enumerate() {
            let color = self.color_scale.sample((i + 1) as f64 / curve_len as f64);
            // middle segments
            canvas.draw_curve_segment(
                |f| middle * 2.0 * f * (1.0 - f)
                    + (start + middle) * f * f / 2.0
                    + (middle + end) * (1.0 - f) * (1.0 - f) / 2.0,
                self.curve_width,
                color,
            );
            if i == curve_len - 2 {
                // last segment
                canvas.draw_curve_segment(
                    |f| interpolate(f, (middle + end) / 2.0, (end * 3.0 - middle) / 2.0),
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
