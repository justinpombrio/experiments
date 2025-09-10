mod arith;
mod canvas;
mod color_data;
mod color_scale;
mod curve;
mod hilbert_3d;
mod oklab;
mod srgb;

use crate::color_data::{CET_L08, CET_L16, CET_L17, CET_L19, CET_RAINBOW};
use argparse::FromCommandLine;
use arith::{interpolate, Bounds, Point};
use canvas::Canvas;
use color_scale::{color_scale_from_data, hilbert_color, hsl, orbit, rgb, sawtooth, scale, Color};
use curve::LindenmayerSystem;

/****************
 * Color Scales *
 ****************/

type ColorScale = fn(f64) -> Color;

const COLOR_SCALES: &[(&str, ColorScale)] = &[
    ("b", rgb_b),
    ("bw", rgb_bw),
    ("bw2", rgb_bw2),
    ("bw3", rgb_bw3),
    ("1", rgb_1),
    ("2", rgb_2),
    ("2s", rgb_2s),
    ("3", rgb_3),
    ("4", rgb_4),
    ("7", rgb_7),
    ("8", rgb_8),
    ("9", rgb_9),
    ("o6", rgb_o6),
    ("h", rgb_hilbert),
    ("ry", rgb_ry),
    ("bg", rgb_bg),
    ("m", rgb_m),
    ("cet-l08", rgb_cet_l08),
    ("cet-l16", rgb_cet_l16),
    ("cet-l17", rgb_cet_l17),
    ("cet-l19", rgb_cet_l19),
    ("cet-rainbow", rgb_cet_rainbow),
];

fn rgb_b(_f: f64) -> Color {
    hsl([0.0, 0.0, 0.25])
}

fn rgb_bw(f: f64) -> Color {
    let hue = 0.0;
    let sat = 0.0;
    let lit = scale(sawtooth(scale(f, 0.5, 1.0)), 0.25, 0.95);
    hsl([hue, sat, lit])
}

fn rgb_bw2(f: f64) -> Color {
    let hue = 0.0;
    let sat = 0.0;
    let lit = scale(sawtooth(scale(f, 0.375, 1.375)), 0.25, 0.95);
    hsl([hue, sat, lit])
}

fn rgb_bw3(f: f64) -> Color {
    let hue = 0.0;
    let sat = 0.0;
    let lit = scale(sawtooth(scale(f, 0.0, 1.0)), 0.25, 0.95);
    hsl([hue, sat, lit])
}

fn rgb_1(f: f64) -> Color {
    let hue = f;
    let sat = 0.176;
    let lit = 0.75;
    hsl([hue, sat, lit])
}

fn rgb_2(f: f64) -> Color {
    let hue = f;
    let sat = 0.175;
    let lit = scale(sawtooth(scale(f, 0.5, 2.0)), 0.25, 0.75);
    hsl([hue, sat, lit])
}

fn rgb_2s(f: f64) -> Color {
    let hue = f;
    let sat = 0.175;
    let lit = scale(sawtooth(scale(sawtooth(f), 0.5, 2.0)), 0.25, 0.75);
    hsl([hue, sat, lit])
}

fn rgb_3(f: f64) -> Color {
    let hue = f;
    let sat = 0.175;
    let lit = scale(sawtooth(scale(f, 0.0, 6.0)), 0.30, 0.70);
    hsl([hue, sat, lit])
}

fn rgb_4(f: f64) -> Color {
    let hue = scale(f, -0.125, 0.875);
    let sat = 0.175;
    let lit = scale(sawtooth(scale(f, 0.375, 1.375)), 0.25, 0.75);
    hsl([hue, sat, lit])
}

fn rgb_7(f: f64) -> Color {
    let hue = scale(f, 0.2, 2.7);
    let sat = 0.175;
    let lit = scale(sawtooth(scale(f, 0.5, 6.5)), 0.40, 0.75);
    hsl([hue, sat, lit])
}

fn rgb_8(f: f64) -> Color {
    let hue = f;
    let sat = 0.175;
    let lit = 0.75 * scale(sawtooth(scale(f, 0.5, 4.5)), 0.003, 1.0).powf(1.0 / 3.0);
    hsl([hue, sat, lit])
}

fn rgb_9(f: f64) -> Color {
    let hue = f;
    let sat = 0.175 * scale(sawtooth(scale(f, 0.0, 9.0)), 0.3, 1.0).powf(1.0 / 2.0);
    let lit = scale(sawtooth(scale(f, 0.5, 5.5)), 0.40, 0.75);
    hsl([hue, sat, lit])
}

fn rgb_o6(f: f64) -> Color {
    let (lit, hue) = orbit(f, (0.0, 1.0, 0.6), (0.0, 6.0, 0.15));
    let sat = 0.175;
    hsl([hue, sat, lit])
}

fn rgb_hilbert(f: f64) -> Color {
    hilbert_color(f)
}

fn rgb_ry(f: f64) -> Color {
    let r = sawtooth(scale(f, 0.0, 0.5));
    let g = sawtooth(scale(f, 0.5, 1.5));
    let b = scale(sawtooth(scale(f, 0.0, 4.0)), 0.0, 0.4);
    rgb([r, g, b])
}

fn rgb_bg(f: f64) -> Color {
    let r = scale(sawtooth(scale(f, 0.0, 4.0)), 0.0, 0.4);
    let g = sawtooth(scale(f, 0.0, 0.5));
    let b = sawtooth(scale(f, 0.5, 1.5));
    rgb([r, g, b])
}

fn rgb_m(f: f64) -> Color {
    let r = scale(sawtooth(scale(f, 0.0, 2.5)), 0.0, 0.6);
    let g = scale(sawtooth(scale(f, 0.0, 3.5)), 0.0, 1.0);
    let b = scale(sawtooth(scale(f, 0.0, 1.5)), 0.0, 1.0);
    rgb([r, g, b])
}

fn rgb_cet_l08(f: f64) -> Color {
    color_scale_from_data(f, CET_L08)
}

fn rgb_cet_l16(f: f64) -> Color {
    color_scale_from_data(f, CET_L16)
}

fn rgb_cet_l17(f: f64) -> Color {
    color_scale_from_data(sawtooth(f), CET_L17)
}

fn rgb_cet_l19(f: f64) -> Color {
    color_scale_from_data(f, CET_L19)
}

fn rgb_cet_rainbow(f: f64) -> Color {
    color_scale_from_data(f, CET_RAINBOW)
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
        "moore",
        LindenmayerSystem {
            start: "-BfB+f+BfB",
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
        "wunderlich",
        LindenmayerSystem {
            start: "S",
            rules: &[('S', "S+fSf-Sf-Sf-S+fS+fS+fSf-S")],
            angle: 90.0,
            implicit_f: false,
        },
    ),
    (
        "sierpinski",
        LindenmayerSystem {
            start: "--A",
            rules: &[('A', "B-A-B"), ('B', "A+B+A")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        "square",
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
    (
        // Improper. Self intersects.
        "triangle",
        LindenmayerSystem {
            start: "L",
            rules: &[('L', "-R+L++R--L"), ('R', "+L-R--L++R")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        // Improper. Self intersects.
        "fivefold",
        LindenmayerSystem {
            start: "X",
            rules: &[('X', "+X-X--XX++X+X-")],
            angle: 60.0,
            implicit_f: true,
        },
    ),
    (
        "s-curve",
        LindenmayerSystem {
            start: "++S",
            rules: &[('S', "+S----S++++S-")],
            angle: 30.0,
            implicit_f: true,
        },
    ),
    (
        // A Plane Filling Curve for the Year 2017
        // https://www.cut-the-knot.org/do_you_know/SpaceFillingArioni.shtml
        "arioni",
        LindenmayerSystem {
            start: "R",
            rules: &[('R', "-QR+R+Q-R"), ('Q', "Q+R-Q-QR+")],
            angle: 90.0,
            implicit_f: true,
        },
    ),
    (
        // https://cl.pinterest.com/pin/pin-auf-spacefilling-curves-2--418201515408899768/
        "steemann",
        LindenmayerSystem {
            start: "R",
            rules: &[
                ('R', "RfL++fLfR--fRfLfR--fRfL++"),
                ('L', "LfR--fRfL++fLfRfL++fLfR--"),
            ],
            angle: 60.0,
            implicit_f: false,
        },
    ),
    // TODO: Pentaflake
    // https://mathworld.wolfram.com/Pentaflake.html
];

/********
 * Main *
 ********/

fn main() {
    use argparse::{ArgumentParser, Parse, Store};
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
    let mut image_name = "curve.png".to_owned();
    // background
    let mut background = Color::from_argument("d2d2d2").unwrap();
    let mut checkers = 0;
    let mut checkers_color_1 = Color::from_argument("dcdcaa").unwrap();
    let mut checkers_color_2 = Color::from_argument("b4c8f0").unwrap();
    // border
    let mut border_width = 0;
    let mut border_color = Color::from_argument("d2d2d2").unwrap();

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
        // Background
        args.refer(&mut background).add_option(
            &["--bg", "--background-color"],
            Parse,
            "Make the background color this hex color (default #d2d2d2)",
        );
        args.refer(&mut checkers).add_option(
            &["--checkerboard"],
            Store,
            "Make the background an 2^N x 2^N checkerboard (default 0, which is off)",
        );
        args.refer(&mut checkers_color_1).add_option(
            &["--checkers-foreground"],
            Parse,
            "The color of half the checker squares, if --checkerboard is set",
        );
        args.refer(&mut checkers_color_2).add_option(
            &["--checkers-foreground"],
            Parse,
            "The color of half the checker squares, if --checkerboard is set",
        );
        // Border
        args.refer(&mut border_width).add_option(
            &["--border"],
            Store,
            "Width of the border (default 0)",
        );
        args.refer(&mut border_color).add_option(
            &["--border-color"],
            Parse,
            "Color of the border (default 'd2d2d2'). You must set border_width to have a border.",
        );
        args.parse_args_or_exit();
    }

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

    let num_points = curve.expand(depth).size_hint().0;
    println!("Drawing {depth} iterations of {curve_name} curve ({num_points} points).");

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
    canvas.fill(border_color);
    if checkers > 0 {
        canvas.draw_checkerboard(
            image_bounds,
            Point {
                x: 2_u32.pow(checkers),
                y: 2_u32.pow(checkers),
            },
            checkers_color_1,
            checkers_color_2,
        );
    } else {
        canvas.draw_rect(image_bounds, background);
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
            let color = color_scale(i as f64 / curve_len as f64);
            canvas.draw_point(point, color);
        }
    } else {
        let mut start = points.next().unwrap();
        let mut middle = points.next().unwrap();
        // first segment
        canvas.draw_curve(
            |f| interpolate(f, (start * 3.0 - middle) / 2.0, (start + middle) / 2.0),
            curve_width,
            color_scale(0.0),
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
                color_scale((i + 1) as f64 / curve_len as f64),
            );
            if i == curve_len - 2 {
                // last segment
                canvas.draw_curve(
                    |f| interpolate(f, (middle + end) / 2.0, (end * 3.0 - middle) / 2.0),
                    curve_width,
                    color_scale(1.0),
                );
            }
            start = middle;
            middle = end;
        }
    }

    println!("Saving to '{}'.", image_name);
    canvas.save(&image_name);
}
