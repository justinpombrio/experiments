mod arith;
mod canvas;
mod curve;
mod oklab;

use arith::{interpolate, Bounds, Point};
use canvas::Canvas;
use curve::LindenmayerSystem;
use oklab::{oklab_hsv_to_srgb, Color};

/*********************
 * Background Colors *
 *********************/

const BORDER_COLOR: Color = [180 * 256, 180 * 256, 180 * 256];
const BACKGROUND_COLOR: Color = [210 * 256, 210 * 256, 210 * 256];
const BACKGROUND_COLOR_BW: Color = [170 * 256, 200 * 256, 250 * 256];
const CHECKERBOARD_COLOR_1: Color = [220 * 256, 220 * 256, 170 * 256];
const CHECKERBOARD_COLOR_2: Color = [180 * 256, 200 * 256, 240 * 256];

/****************
 * Color Scales *
 ****************/

type ColorScale = fn(f64) -> [f64; 3];

const COLOR_SCALES: &[(&str, ColorScale)] = &[
    ("bw", hsv_bw),
    ("bw2", hsv_bw2),
    ("1", hsv_1),
    ("2", hsv_2),
    ("3", hsv_3),
    ("4", hsv_4),
    ("7", hsv_7),
    ("8", hsv_8),
    ("9", hsv_9),
    ("o4", hsv_o4),
];

fn hsv_bw(f: f64) -> [f64; 3] {
    let hue = 0.0;
    let sat = 0.0;
    let val = linear_cycle(f, (0.5, 1.0), (0.25, 0.95));
    [hue, sat, val]
}

fn hsv_bw2(f: f64) -> [f64; 3] {
    let hue = 0.0;
    let sat = 0.0;
    let val = linear_cycle(f, (0.375, 1.375), (0.25, 0.95));
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

fn hsv_4(f: f64) -> [f64; 3] {
    let hue = cycle(f, -0.125, 0.875);
    let sat = 0.175;
    let val = linear_cycle(f, (0.375, 1.375), (0.25, 0.75));
    [hue, sat, val]
}

fn hsv_8(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.175;
    let val = 0.75 * linear_cycle(f, (0.5, 4.5), (0.003, 1.0)).powf(1.0 / 3.0);
    [hue, sat, val]
}

fn hsv_9(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.0, 1.0);
    let sat = 0.175 * linear_cycle(f, (0.0, 9.0), (0.3, 1.0)).powf(1.0 / 2.0);
    let val = linear_cycle(f, (0.5, 5.5), (0.40, 0.75));
    [hue, sat, val]
}

fn hsv_7(f: f64) -> [f64; 3] {
    let hue = cycle(f, 0.2, 2.7);
    let sat = 0.175;
    let val = linear_cycle(f, (0.5, 6.5), (0.40, 0.75));
    [hue, sat, val]
}

fn hsv_o4(f: f64) -> [f64; 3] {
    let (val, hue) = orbit(f, (0.0, 1.0, 0.6), (0.0, 4.0, 0.15));
    let sat = 0.175;
    [hue, sat, val]
}

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

fn orbit(
    f: f64,
    (big_start, big_end, big_rad): (f64, f64, f64),
    (little_start, little_end, little_rad): (f64, f64, f64),
) -> (f64, f64) {
    let big_vec = Point::cis(interpolate(f, big_start, big_end)) * big_rad;
    let little_vec = Point::cis(interpolate(f, little_start, little_end)) * little_rad;
    let vector = big_vec + little_vec;
    (vector.abs(), vector.angle())
}

/**********
 * Curves *
 **********/

const CURVES: &[(&str, LindenmayerSystem)] = &[
    (
        "hilbert",
        LindenmayerSystem {
            start: "A",
            rules: &[('A', "+Bf-AfA-fB+"), ('B', "-Af+BfB+fA-")],
            angle: 90.0,
            implicit_f: false,
        },
    ),
    (
        "zorder",
        LindenmayerSystem {
            start: "Z",
            rules: &[('Z', "ZzZzZzZ")],
            angle: 0.0,
            implicit_f: false,
        },
    ),
    (
        "dragon",
        LindenmayerSystem {
            start: "R",
            rules: &[('R', "Rf+L"), ('L', "Rf-L")],
            angle: 90.0,
            implicit_f: false,
        },
    ),
    (
        "gosper",
        LindenmayerSystem {
            start: "A",
            rules: &[('A', "A-B--B+A++AA+B-"), ('B', "+A-BB--B-A++A+B")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        "peano",
        LindenmayerSystem {
            start: "+L",
            rules: &[
                ('L', "LfRfL-f-RfLfR+f+LfRfL"),
                ('R', "RfLfR+f+LfRfL-f-RfLfR"),
            ],
            angle: 90.0,
            implicit_f: false,
        },
    ),
    (
        "sierpinski",
        LindenmayerSystem {
            start: "-f++Xf++f++Xf",
            rules: &[('X', "Xf--f++f--Xf++f++Xf--f++f--X")],
            angle: 45.0,
            implicit_f: false,
        },
    ),
    (
        "koch",
        LindenmayerSystem {
            start: "X",
            rules: &[('X', "X-X++X-X")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        "koch-90",
        LindenmayerSystem {
            start: "X",
            rules: &[('X', "X-X+X+X-X")],
            angle: 90.0,
            implicit_f: true,
        },
    ),
    // Improper. Self intersects.
    (
        "triangle",
        LindenmayerSystem {
            start: "L",
            rules: &[('L', "-R+L++R--L"), ('R', "+L-R--L++R")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        "fivefold",
        LindenmayerSystem {
            start: "X",
            rules: &[('X', "+X-X--XX++X+X-")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
];

/********
 * Main *
 ********/

fn main() {
    use argparse::{ArgumentParser, Store};
    let curve_name_options = CURVES
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ");
    let color_scale_options = COLOR_SCALES
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(", ");

    // Options to be set
    let mut curve_name = "hilbert".to_owned();
    let mut depth = 3;
    let mut curve_width = 0.5;
    let mut color_scale_name = "bw".to_owned();
    let mut image_size = 1024;
    let mut border_width = 0;
    let mut image_name = "curve.png".to_owned();
    let mut checkers = 0;

    // Parse command line args (sets the above options)
    {
        let curve_name_description =
            format!("Which curve to use. Options are: {}.", curve_name_options);
        let color_scale_description = format!(
            "Which color scale to use (default bw). Options are: {}.",
            color_scale_options
        );

        let mut args = ArgumentParser::new();
        args.set_description("Draw fancy curves.");
        args.refer(&mut curve_name)
            .add_argument("curve", Store, &curve_name_description)
            .required();
        args.refer(&mut depth)
            .add_argument(
                "iterations",
                Store,
                "How many iterations to repeat the curve for.",
            )
            .required();
        args.refer(&mut curve_width)
            .add_option(&["-t", "--thickness"], Store,
                "How wide the curve should be, where 1.0 is thick enought to touch itself (default 0.5). Exactly 0 draws individual points.");
        args.refer(&mut color_scale_name).add_option(
            &["-c", "--colors"],
            Store,
            &color_scale_description,
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

    println!("Drawing {} iterations of {} curve.", depth, curve_name);

    // Look up color scale, or error
    let color_scale = {
        let mut found = None;
        for (name, scale) in COLOR_SCALES {
            if &color_scale_name == name {
                found = Some(scale);
            }
        }
        match found {
            Some(scale) => scale,
            None => panic!(
                "Color scale name '{}' not recognized. Options are {}",
                color_scale_name, color_scale_options
            ),
        }
    };

    // Look up curve name, or error
    let curve = {
        let mut found = None;
        for (name, curve) in CURVES {
            if &curve_name == name {
                found = Some(curve);
            }
        }
        match found {
            Some(curve) => curve,
            None => panic!(
                "Curve name '{}' not recognized. Options are {}",
                curve_name, curve_name_options
            ),
        }
    };

    // Determine bounds of the curve by walking it
    let bounds = {
        let bounds = curve.bounds(depth);
        let center = (bounds.min + bounds.max) / 2.0;
        let dimensions = bounds.max - bounds.min;
        let new_dimensions = Point::zero() + dimensions.x.max(dimensions.y);
        Bounds {
            min: center - new_dimensions / 2.0 - 0.5,
            max: center + new_dimensions / 2.0 + 0.5,
        }
    };

    // Calculate image bounds (just based on size & border)
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

    // Start drawing! Make a canvas.
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

    // Draw the curve itself
    let drawing_size = bounds.max - bounds.min;
    let points = curve.expand(depth);
    let curve_len = points.len() - 1;
    let canvas_size = Point {
        x: (image_size - 2 * border_width) as f64,
        y: (image_size - 2 * border_width) as f64,
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
        canvas.draw_curve(
            |f| interpolate(f, (start * 3.0 - middle) / 2.0, (start + middle) / 2.0),
            curve_width,
            oklab_hsv_to_srgb(color_scale(0.0)).expect("Color out of bounds"),
        );
        for (i, end) in points.enumerate() {
            // middle segments
            canvas.draw_curve(
                |f| {
                    middle * 2.0 * f * (1.0 - f)
                        + (start + middle) * f * f / 2.0
                        + (middle + end) * (1.0 - f) * (1.0 - f) / 2.0
                },
                curve_width,
                oklab_hsv_to_srgb(color_scale((i + 1) as f64 / curve_len as f64))
                    .expect("Color out of bounds"),
            );
            if i == curve_len - 2 {
                // last segment
                canvas.draw_curve(
                    |f| interpolate(f, (middle + end) / 2.0, (end * 3.0 - middle) / 2.0),
                    curve_width,
                    oklab_hsv_to_srgb(color_scale(1.0)).expect("Color out of bounds"),
                );
            }
            start = middle;
            middle = end;
        }
    }

    println!("Saving to '{}'.", image_name);
    canvas.save(&image_name);
}
