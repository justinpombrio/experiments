mod curve;
mod hilbert_curve;
mod oklab;

use curve::HILBERT_CURVE;
use hilbert_curve::HilbertCurve;
use image::{ImageBuffer, Rgb};
use oklab::oklab_to_srgb;
use std::mem;

const COLORS: ColorScaleConfig = ColorScaleConfig {
    hue_range: (0.0, 1.0),
    shade_range: (0.0, 3.0),
    min_shade: 0.3,
    saturation: 0.127,
    lightness: 0.75,
};

// OkLab Colors. All angles are measured in turns. Obviously.
const BORDER_COLOR: [u16; 3] = [u16::MAX / 3, u16::MAX / 3, u16::MAX / 3];
const HACKY_SAT_MULTIPLIER: f64 = 1.15;

// Dimensions
const IMAGE_SIZE: u32 = 4096;
const BORDER_SIZE: u32 = 32;
const HILBERT_ITERS: u32 = 8;
const HILBERT_SIZE: u32 = 2_u32.pow(HILBERT_ITERS);
const HILBERT_CURVE_LEN: u32 = HILBERT_SIZE * HILBERT_SIZE;
const CELL_WIDTH: u32 = IMAGE_SIZE / HILBERT_SIZE;

// Squares
const DRAW_SQUARES: bool = true;

// Curve
const DRAW_CURVE: bool = false;
const LINE_WIDTH: u32 = 7 * CELL_WIDTH / 8;
const USE_FIXED_CURVE_COLOR: bool = true;
const CURVE_COLOR: [u16; 3] = [30 * 256, 30 * 256, 30 * 256];
const ENDPOINT_BORDER: u32 = CELL_WIDTH / 16;

// Checkerboard
const DRAW_CHECKERBOARD: bool = false;
const FOREGROUND_COLOR: [u16; 3] = [u16::MAX, u16::MAX, u16::MAX];
const BACKGROUND_COLOR: [u16; 3] = [0, 0, 0];

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Color = [u16; 3];

pub fn main() {
    let mut img =
        ImageBuffer::<Rgb<u16>, _>::new(IMAGE_SIZE + 2 * BORDER_SIZE, IMAGE_SIZE + 2 * BORDER_SIZE);

    // Draw background
    for pixel in img.pixels_mut() {
        pixel.0 = BORDER_COLOR;
    }
    if DRAW_CHECKERBOARD {
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let x = x + BORDER_SIZE;
            let y = y + BORDER_SIZE;
            let sx = x / CELL_WIDTH / 8;
            let sy = y / CELL_WIDTH / 8;
            if (sx + sy) % 2 == 0 {
                pixel.0 = FOREGROUND_COLOR;
            } else {
                pixel.0 = BACKGROUND_COLOR;
            }
        }
    }

    let scale = |point: (f64, f64)| -> (u32, u32) {
        (
            CELL_WIDTH * point.0.round() as u32 + CELL_WIDTH / 2,
            CELL_WIDTH * point.1.round() as u32 + CELL_WIDTH / 2,
        )
    };

    // Draw square background
    //let curve = HilbertCurve::new(HILBERT_SIZE);
    let mut total_colors = 0;
    let mut clamped_colors = 0;
    if DRAW_SQUARES {
        let curve = HILBERT_CURVE.expand(HILBERT_ITERS as usize);
        for (d, point) in curve.enumerate() {
            let point = scale(point);
            let frac = (d as f64) / (HILBERT_CURVE_LEN as f64);
            let (color, clamped) = color_scale(frac, COLORS);
            total_colors += 1;
            clamped_colors += clamped as u32;
            let lower_left = (point.0 - CELL_WIDTH / 2, point.1 - CELL_WIDTH / 2);
            let upper_right = (point.0 + CELL_WIDTH / 2, point.1 + CELL_WIDTH / 2);
            draw_rect(&mut img, lower_left, upper_right, color);
        }
    }
    println!(
        "Fraction of colors that had to be clamped: {}",
        clamped_colors as f32 / total_colors as f32
    );

    // Draw curve
    if DRAW_CURVE {
        let mut curve = HILBERT_CURVE.expand(HILBERT_ITERS as usize);
        let mut start = scale(curve.next().unwrap());
        let mut middle = scale(curve.next().unwrap());
        for (d, point) in curve.enumerate() {
            let end = scale(point);
            let frac = (d as f64) / (HILBERT_CURVE_LEN as f64);
            let color = if USE_FIXED_CURVE_COLOR {
                CURVE_COLOR
            } else {
                color_scale(frac, COLORS).0
            };
            draw_segment(&mut img, start, middle, end, LINE_WIDTH / 2, color);
            start = middle;
            middle = end;
        }
    }

    // Draw start and end points
    let start_color = color_scale(0.0, COLORS).0;
    let end_color = color_scale(1.0, COLORS).0;
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let width = CELL_WIDTH - ENDPOINT_BORDER;
        let border = ENDPOINT_BORDER;
        let flipped_x = IMAGE_SIZE + 2 * BORDER_SIZE - x - 1;
        let x = x + BORDER_SIZE;
        let y = y + BORDER_SIZE;
        if x < width && x >= border && y < width && y >= border {
            pixel.0 = start_color;
        }
        if flipped_x < width && flipped_x >= border && y < width && y >= border {
            pixel.0 = end_color;
        }
    }

    println!("Saving image to hilbert.png");
    img.save("hilbert.png").unwrap();
}

fn draw_rect(
    img: &mut Image,
    mut lower_left: (u32, u32),
    mut upper_right: (u32, u32),
    color: Color,
) {
    if lower_left.0 > upper_right.0 {
        mem::swap(&mut lower_left.0, &mut upper_right.0);
    }
    if lower_left.1 > upper_right.1 {
        mem::swap(&mut lower_left.1, &mut upper_right.1);
    }
    for x in lower_left.0..upper_right.0 {
        for y in lower_left.1..upper_right.1 {
            img.get_pixel_mut(x + BORDER_SIZE, y + BORDER_SIZE).0 = color;
        }
    }
}

fn draw_segment(
    img: &mut Image,
    start: (u32, u32),
    middle: (u32, u32),
    end: (u32, u32),
    width: u32,
    color: Color,
) {
    if start.0 == end.0 {
        let lower_left = (start.0 - width, (start.1 + middle.1) / 2);
        let upper_right = (start.0 + width, (middle.1 + end.1) / 2);
        draw_rect(img, lower_left, upper_right, color);
    } else if start.1 == end.1 {
        let lower_left = ((start.0 + middle.0) / 2, start.1 - width);
        let upper_right = ((middle.0 + end.0) / 2, start.1 + width);
        draw_rect(img, lower_left, upper_right, color);
    } else {
        let radius = (CELL_WIDTH / 2) as f64;
        let center = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
        let dmiddle = (2 * middle.0 - center.0, 2 * middle.1 - center.1);
        for x in center.0.min(dmiddle.0)..center.0.max(dmiddle.0) {
            for y in center.1.min(dmiddle.1)..center.1.max(dmiddle.1) {
                let dx = x as f64 - center.0 as f64;
                let dy = y as f64 - center.1 as f64;
                let dist = (dx * dx + dy * dy).sqrt();
                if (dist - radius).abs() < width as f64 {
                    img.get_pixel_mut(x, y).0 = color;
                }
            }
        }
    }
}

struct ColorScaleConfig {
    hue_range: (f64, f64),
    shade_range: (f64, f64),
    min_shade: f64,
    saturation: f64,
    lightness: f64,
}

fn color_scale(f: f64, config: ColorScaleConfig) -> (Color, bool) {
    const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

    fn interpolate(f: f64, (start, end): (f64, f64)) -> f64 {
        start + f * (end - start)
    }

    let angle = interpolate(f, config.hue_range);
    let shade = interpolate(f, config.shade_range);
    let shade = ((shade % 2.0) - 1.0).abs();
    let shade = shade * (1.0 - config.min_shade) + config.min_shade;
    let saturation = shade * config.saturation;
    let l = shade * config.lightness;
    let a = saturation * (RADS_PER_TURN * angle).sin();
    let b = saturation * (RADS_PER_TURN * angle).cos();
    oklab_to_srgb([l, a, b])
}
