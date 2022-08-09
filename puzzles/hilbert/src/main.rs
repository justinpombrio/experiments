mod arith;
mod canvas;
mod curve;
mod oklab;

use arith::{interpolate, Bounds, Point};
use canvas::Canvas;
use curve::LindenmayerSystem;
use oklab::{Color, ColorScale};

const BORDER_COLOR: Color = [200 * 256, 200 * 256, 200 * 256];
const BACKGROUND_COLOR: Color = [u16::MAX, u16::MAX, u16::MAX];
const CHECKERBOARD_COLOR_1: Color = [220 * 256, 220 * 256, 170 * 256];
const CHECKERBOARD_COLOR_2: Color = [180 * 256, 200 * 256, 240 * 256];

const HILBERT_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "A",
    rules: &[('A', "rBflAfAlfBr"), ('B', "lAfrBfBrfAl")],
    len: hilbert_len,
};
fn hilbert_len(depth: usize) -> usize {
    4_usize.pow(depth as u32)
}

fn hilbert_drawing(image_size: u32, curve_width: f64, depth: usize) -> Drawing {
    Drawing {
        curve: HILBERT_CURVE,
        depth,
        curve_width: curve_width * image_size as f64 / 2_u32.pow(depth as u32) as f64 * 0.5,
        bounds: Bounds {
            min: Point { x: -0.5, y: -0.5 },
            max: Point {
                x: 2_u32.pow(depth as u32) as f64 - 0.5,
                y: 2_u32.pow(depth as u32) as f64 - 0.5,
            },
        },
        color_scale: ColorScale {
            max_saturation: 0.127,
            min_lightness: 0.25,
            max_lightness: 0.75,
            hsv: color_scale_2,
        },
    }
}

fn color_scale_2(f: f64) -> (f64, f64, f64) {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 1.0;
    let val = linear_cycle(f, 0.5, 2.0, 0.0);
    (hue, sat, val)
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
        let mut points =
            points.map(move |point| (point - self.bounds.min) / drawing_size * canvas_size);

        let mut start = points.next().unwrap();
        let mut middle = points.next().unwrap();
        // first segment
        canvas.draw_curve_segment(
            |f| interpolate(f, (start * 3.0 - middle) / 2.0, (start + middle) / 2.0),
            self.curve_width,
            self.color_scale.sample(0.0),
        );
        for (i, end) in points.enumerate() {
            let color = self.color_scale.sample((i + 1) as f64 / curve_len as f64);
            // middle segments
            canvas.draw_curve_segment(
                |f| {
                    middle * 2.0 * f * (1.0 - f)
                        + (start + middle) * f * f / 2.0
                        + (middle + end) * (1.0 - f) * (1.0 - f) / 2.0
                },
                self.curve_width,
                color,
            );
            if i == curve_len - 2 {
                // last segment
                canvas.draw_curve_segment(
                    |f| interpolate(f, (middle + end) / 2.0, (end * 3.0 - middle) / 2.0),
                    self.curve_width,
                    self.color_scale.sample(1.0),
                );
            }
            start = middle;
            middle = end;
        }
    }
}

fn main() {
    use argparse::{ArgumentParser, Store};

    let mut curve_name = "hilbert".to_owned();
    let mut depth = 3;
    let mut curve_width = 0.5;
    let mut image_size = 1024;
    let mut border_width = 0;
    let mut image_name = "curve.png".to_owned();
    let mut checkers = 0;

    {
        let mut args = ArgumentParser::new();
        args.set_description("Draw fancy curves.");
        args.refer(&mut curve_name)
            .add_argument(
                "curve",
                Store,
                "Which curve to use (default hilbert). Options are hilbert, peano, morton, dragon.",
            )
            .required();
        args.refer(&mut depth).add_option(
            &["-i", "--iters"],
            Store,
            "How many iterations to repeat the curve for.",
        );
        args.refer(&mut curve_width)
            .add_option(&["-t", "--thickness"], Store,
                "How wide the curve should be, where 1.0 is thick enought to touch itself (default 0.5). Exactly 0 draws individual points.");
        args.refer(&mut border_width).add_option(
            &["-b", "--border"],
            Store,
            "Width of the border (default 0)",
        );
        args.refer(&mut checkers).add_option(
            &["-h", "--checkerboard"],
            Store,
            "Make the background an 2^N x 2^N checkerboard (default 0, which is off)",
        );
        args.refer(&mut image_size).add_option(
            &["-s", "--size"],
            Store,
            "Width&height of image (default 1024)",
        );
        args.refer(&mut image_name).add_option(
            &["-o", "--output"],
            Store,
            "File name of output image (default 'curve.png')",
        );
        args.parse_args_or_exit();
    }

    let drawing = match curve_name.as_ref() {
        "hilbert" => hilbert_drawing(image_size, curve_width, depth),
        name => panic!("Curve name '{}' not recognized", name),
    };
    let image_bounds = Bounds {
        min: Point {
            x: border_width,
            y: border_width,
        },
        max: Point {
            x: image_size - border_width,
            y: image_size - border_width,
        },
    };

    let mut canvas = Canvas::new(image_size, image_size);
    canvas.fill(BORDER_COLOR);
    if checkers > 0 {
        canvas.draw_checkerboard(
            image_bounds,
            Point {
                x: 2_u32.pow(checkers),
                y: 2_u32.pow(checkers),
            },
            CHECKERBOARD_COLOR_1,
            CHECKERBOARD_COLOR_2,
        );
    } else {
        canvas.draw_rect(image_bounds, BACKGROUND_COLOR);
    }
    drawing.draw_on_canvas(&mut canvas);
    canvas.save("curve.png");
}
