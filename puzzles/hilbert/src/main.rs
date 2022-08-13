mod arith;
mod canvas;
mod curve;
mod oklab;

use arith::{interpolate, Bounds, Point};
use canvas::Canvas;
use curve::LindenmayerSystem;
use oklab::{Color, oklab_hsv_to_srgb};

/**********
 * Colors *
 **********/

const BORDER_COLOR: Color = [180 * 256, 180 * 256, 180 * 256];
const BACKGROUND_COLOR: Color = [210 * 256, 210 * 256, 210 * 256];
const BACKGROUND_COLOR_BW: Color = [170 * 256, 200 * 256, 250 * 256];
const CHECKERBOARD_COLOR_1: Color = [220 * 256, 220 * 256, 170 * 256];
const CHECKERBOARD_COLOR_2: Color = [180 * 256, 200 * 256, 240 * 256];

/****************
 * Color Scales *
 ****************/

fn hsv_bw(f: f64) -> [f64; 3] {
    let hue = 0.0;
    let sat = 0.0;
    let val = linear_cycle(f, (0.5, 1.0), (0.25, 0.95));
    [hue, sat, val]
}

fn hsv_1(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.176;
    let val = 0.75;
    [hue, sat, val]
}

fn hsv_2(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.175;
    let val = linear_cycle(f, (0.5, 2.0), (0.25, 0.75));
    [hue, sat, val]
}

fn hsv_3(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.175;
    let val = linear_cycle(f, (0.0, 6.0), (0.30, 0.70));
    [hue, sat, val]
}

fn hsv_8(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.175;
    let val = 0.75 * linear_cycle(f, (0.5, 4.5), (0.003, 1.0)).powf(1.0 / 3.0);
    [hue, sat, val]
}

fn hsv_9(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 2.0);
    let sat = 0.175 * linear_cycle(f, (0.0, 9.0), (0.2, 1.0)).powf(1.0 / 2.0);
    let val = linear_cycle(f, (0.0, 5.0), (0.30, 0.70));
    [hue, sat, val]
}

/*
const COLOR_SCALE_3: ColorScale = ColorScale {
    max_saturation: 0.176,
    max_lightness: 0.75,
    hsv: hsv_3,
};

fn hsv_3(f: f64) -> (f64, f64, f64) {
    let hue = cycle(f, 0.0, 1.0);
    let sat = linear_cycle(f, 0.5, 3.5, 0.0);
    let val = linear_cycle(f, 0.5, 2.0, 0.3);
    (hue, sat.powf(1.0/3.0), val)
}

const COLOR_SCALE_7: ColorScale = ColorScale {
    max_saturation: 0.176,
    max_lightness: 0.75,
    hsv: hsv_7,
};

fn hsv_7(f: f64) -> (f64, f64, f64) {
    let hue = cycle(f, 0.0, 5.0);
    let sat = linear_cycle(f, 0.0, 6.0, 0.5);
    let val = linear_cycle(f, 0.0, 7.0, 0.5);
    (hue, sat, val)
}
*/

/*****************
 * Hilbert Curve *
 *****************/

const HILBERT_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "A",
    rules: &[('A', "rBflAfAlfBr"), ('B', "lAfrBfBrfAl")],
    len: hilbert_len,
};
fn hilbert_len(depth: usize) -> usize {
    4_usize.pow(depth as u32)
}
fn hilbert_bounds(depth: usize) -> Bounds<f64> {
    let min = -0.5;
    let max = 2_u32.pow(depth as u32) as f64 - 0.5;
    Bounds {
        min: Point { x: min, y: min },
        max: Point { x: max, y: max }
    }
}

/*****************
 * Z-Order Curve *
 *****************/

const Z_ORDER_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "Z",
    rules: &[('Z', "ZzZzZzZ")],
    len: z_order_len,
};
fn z_order_len(depth: usize) -> usize {
    4_usize.pow(depth as u32)
}
fn z_order_bounds(depth: usize) -> Bounds<f64> {
    let min = -0.5;
    let max = 2_u32.pow(depth as u32) as f64 - 0.5;
    Bounds {
        min: Point { x: min, y: min },
        max: Point { x: max, y: max }
    }
}

/****************
 * Dragon Curve *
 ****************/

const DRAGON_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "R",
    rules: &[('R', "RfrL"), ('L', "RflL")],
    len: dragon_len,
};
fn dragon_len(depth: usize) -> usize {
    2_usize.pow(depth as u32)
}
fn dragon_bounds(depth: usize) -> Bounds<f64> {
    let bounds = DRAGON_CURVE.bounds(depth);
    let center = (bounds.min + bounds.max) / 2.0;
    let dimensions = bounds.max - bounds.min;
    let new_dimensions = Point::zero() + dimensions.x.max(dimensions.y);
    Bounds {
        min: center - new_dimensions/2.0 - 0.5,
        max: center + new_dimensions/2.0 + 0.5,
    }
}

/****************
 * Gosper Curve *
 ****************/

const GOSPER_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "A",
    rules: &[('A', "gApgBppgBqgAqqgAgAqgBp"),
             ('B', "qgApgBgBppgBpgAqqgAqgB")],
    len: gosper_len,
};
fn gosper_len(depth: usize) -> usize {
    7_usize.pow(depth as u32)
}
fn gosper_bounds(depth: usize) -> Bounds<f64> {
    let bounds = GOSPER_CURVE.bounds(depth);
    let center = (bounds.min + bounds.max) / 2.0;
    let dimensions = bounds.max - bounds.min;
    let new_dimensions = Point::zero() + dimensions.x.max(dimensions.y);
    Bounds {
        min: center - new_dimensions/2.0 - 0.5,
        max: center + new_dimensions/2.0 + 0.5,
    }
}

/***************
 * Peano Curve *
 ***************/

const PEANO_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "rL",
    rules: &[('L', "LfRfLlflRfLfRrfrLfRfL"),
             ('R', "RfLfRrfrLfRfLlflRfLfR")],
    len: peano_len,
};
fn peano_len(depth: usize) -> usize {
    9_usize.pow(depth as u32)
}
fn peano_bounds(depth: usize) -> Bounds<f64> {
    let bounds = PEANO_CURVE.bounds(depth);
    let center = (bounds.min + bounds.max) / 2.0;
    let dimensions = bounds.max - bounds.min;
    let new_dimensions = Point::zero() + dimensions.x.max(dimensions.y);
    Bounds {
        min: center - new_dimensions/2.0 - 0.5,
        max: center + new_dimensions/2.0 + 0.5,
    }
}

/********************
 * Sierpinski Curve *
 ********************/

const SIERPINSKI_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "frXfrfrXf",
    rules: &[('X', "XflfrflXfrfrXflfrflX")],
    len: sierpinski_len,
};
fn sierpinski_len(depth: usize) -> usize {
    let mut f = 4;
    let mut x = 2;
    for _ in 0..depth {
        f = 8 * x + f;
        x = 4 * x;
    }
    f + 1
}
fn sierpinski_bounds(depth: usize) -> Bounds<f64> {
    let bounds = SIERPINSKI_CURVE.bounds(depth);
    let center = (bounds.min + bounds.max) / 2.0;
    let dimensions = bounds.max - bounds.min;
    let new_dimensions = Point::zero() + dimensions.x.max(dimensions.y);
    Bounds {
        min: center - new_dimensions/2.0 - 0.5,
        max: center + new_dimensions/2.0 + 0.5,
    }
}

/*****************/

/// As `f` scales from 0.0 to 1.0, the result scales from `start` to `end`.
fn cycle(f: f64, start: f64, end: f64) -> f64 {
    (start + f * (end - start)) % 1.0
}

/// As `f` scales from 0.0 to 1.0, the result varies between `c(start)` and `c(end)`,
/// where `c` is a cyclic linear function varying between 0.0 at `0, 1, 2, ...` and 1.0 at
/// `0.5, 1.5, 2.5, ...`.
fn linear_cycle(f: f64, (start, end): (f64, f64), (min, max): (f64, f64)) -> f64 {
    min + (1.0 - (2.0 * cycle(f, start, end) - 1.0).abs()) * (max - min)
}

fn main() {
    use argparse::{ArgumentParser, Store};

    let mut curve_name = "hilbert".to_owned();
    let mut depth = 3;
    let mut curve_width = 0.5;
    let mut color_scale_name = "bw".to_owned();
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
                "Which curve to use (default hilbert). Options are hilbert, peano, zorder, dragon.",
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
        args.refer(&mut color_scale_name).add_option(
            &["-c", "--colors"], Store,
            "Color scale (default 'bw'). Options are bw, 2, 3, 7.",
        );
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

    let color_scale = match color_scale_name.as_ref() {
        "bw" => hsv_bw,
        "1" => hsv_1,
        "2" => hsv_2,
        "3" => hsv_3,
        "8" => hsv_8,
        "9" => hsv_9,
        name => panic!("Color scale name '{}' not recognized", name),
    };
    let (curve, bounds) = match curve_name.as_ref() {
        "hilbert" => (HILBERT_CURVE, hilbert_bounds(depth)),
        "zorder" => (Z_ORDER_CURVE, z_order_bounds(depth)),
        "dragon" => (DRAGON_CURVE, dragon_bounds(depth)),
        "gosper" => (GOSPER_CURVE, gosper_bounds(depth)),
        "peano" => (PEANO_CURVE, peano_bounds(depth)),
        "sierpinski" => (SIERPINSKI_CURVE, sierpinski_bounds(depth)),
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

    // Draw background
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
        let background_color = if color_scale_name == "bw" {
            BACKGROUND_COLOR_BW
        } else {
            BACKGROUND_COLOR
        };
        canvas.draw_rect(image_bounds, background_color);
    }

    let drawing_size = bounds.max - bounds.min;
    let points = curve.expand(depth);
    let curve_len = points.len() - 1;

    let canvas_size = Point {
        x: (canvas.size.x - 2 * border_width) as f64,
        y: (canvas.size.y - 2 * border_width) as f64,
    };
    curve_width *= canvas_size.x / drawing_size.x / 2.0;
    let mut points = points
        .map(move |point| (point - bounds.min) / drawing_size * canvas_size + border_width as f64);

    if curve_width == 0.0 {
        // If curve_width=0, draw just the points
        for (i, point) in points.enumerate() {
            let color = oklab_hsv_to_srgb(color_scale(i as f64 / curve_len as f64))
                .expect("Color out of bounds");
            canvas.draw_point(point, color);
        }
    } else {
        let mut start = points.next().unwrap();
        let mut middle = points.next().unwrap();
        // first segment
        canvas.draw_curve_segment(
            |f| interpolate(f, (start * 3.0 - middle) / 2.0, (start + middle) / 2.0),
            curve_width,
            oklab_hsv_to_srgb(color_scale(0.0)).expect("Color out of bounds")
        );
        for (i, end) in points.enumerate() {
            //let color = color_scale.sample((i + 1) as f64 / curve_len as f64);
            // middle segments
            canvas.draw_curve_segment(
                |f| {
                    middle * 2.0 * f * (1.0 - f)
                        + (start + middle) * f * f / 2.0
                        + (middle + end) * (1.0 - f) * (1.0 - f) / 2.0
                },
                curve_width,
                oklab_hsv_to_srgb(color_scale((i + 1) as f64 / curve_len as f64))
                    .expect("Color out of bounds")
            );
            if i == curve_len - 2 {
                // last segment
                canvas.draw_curve_segment(
                    |f| interpolate(f, (middle + end) / 2.0, (end * 3.0 - middle) / 2.0),
                    curve_width,
                    oklab_hsv_to_srgb(color_scale(1.0)).expect("Color out of bounds")
                );
            }
            start = middle;
            middle = end;
        }
    }

    canvas.save("curve.png");
}
