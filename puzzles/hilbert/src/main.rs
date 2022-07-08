mod hilbert_curve;
mod oklab;

use hilbert_curve::HilbertCurve;
use image::{ImageBuffer, Rgb};
use oklab::oklab_to_srgb;
use std::mem;

// OkLab Colors. All angles are measured in turns. Obviously.
const START_COLOR_ANGLE: f64 = 0.0;
const TOTAL_COLOR_ANGLE: f64 = 1.0;

// Dimensions
const IMAGE_SIZE: u32 = 4096;
const HILBERT_SIZE: u32 = 256;
const CELL_WIDTH: u32 = IMAGE_SIZE / HILBERT_SIZE;

// Squares
const DRAW_SQUARES: bool = true;

// Curve
const DRAW_CURVE: bool = false;
const LINE_WIDTH: u32 = 7 * CELL_WIDTH / 8;
const USE_FIXED_CURVE_COLOR: bool = false;
const CURVE_COLOR: [u16; 3] = [30, 30, 30];
const ENDPOINT_BORDER: u32 = CELL_WIDTH / 16;

// Checkerboard
const DRAW_CHECKERBOARD: bool = false;
const FOREGROUND_COLOR: [u16; 3] = [0, 0, 0];
const BACKGROUND_COLOR: [u16; 3] = [255, 255, 255];

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Color = [u16; 3];

pub fn main() {
    let mut img = ImageBuffer::<Rgb<u16>, _>::new(IMAGE_SIZE, IMAGE_SIZE);

    // Draw background
    for pixel in img.pixels_mut() {
        pixel.0 = BACKGROUND_COLOR;
    }
    if DRAW_CHECKERBOARD {
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let sx = x / CELL_WIDTH / 8;
            let sy = y / CELL_WIDTH / 8;
            if (sx + sy) % 2 == 0 {
                pixel.0 = FOREGROUND_COLOR;
            }
        }
    }

    let scale = |point: (u32, u32)| -> (u32, u32) {
        (CELL_WIDTH * point.0 + CELL_WIDTH / 2, CELL_WIDTH * point.1 + CELL_WIDTH / 2)
    };

    // Draw square background
    let curve = HilbertCurve::new(HILBERT_SIZE);
    let mut total_colors = 0;
    let mut clamped_colors = 0;
    if DRAW_SQUARES {
        for d in 0..curve.length() {
            let point = scale(curve.dist_to_point(d));
            let frac = (d as f64) / (curve.length() as f64);
            let (color, clamped) = colorscale(frac, 0.0, 1.0);
            total_colors += 1;
            clamped_colors += clamped as u32;
            let lower_left = (point.0 - CELL_WIDTH / 2, point.1 - CELL_WIDTH / 2);
            let upper_right = (point.0 + CELL_WIDTH / 2, point.1 + CELL_WIDTH / 2);
            draw_rect(&mut img, lower_left, upper_right, color);
        }
    }
    println!("Fraction of colors that had to be clamped: {}",
             clamped_colors as f32 / total_colors as f32);

    // Draw curve
    if DRAW_CURVE {
        for d in 2..curve.length() {
            let start = scale(curve.dist_to_point(d - 2));
            let middle = scale(curve.dist_to_point(d - 1));
            let end = scale(curve.dist_to_point(d));
            let frac = (d as f64) / (curve.length() as f64);
            let color = if USE_FIXED_CURVE_COLOR {
                CURVE_COLOR
            } else {
                colorscale(frac, 0.0, 1.0).0
            };
            draw_segment(&mut img, start, middle, end, LINE_WIDTH / 2, color);
        }
    }

    // Draw start and end points
    let start_color = colorscale(0.0, 0.0, 1.0).0;
    let end_color = colorscale(1.0, 0.0, 1.0).0;
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let width = CELL_WIDTH - ENDPOINT_BORDER;
        let border = ENDPOINT_BORDER;
        let flipped_x = IMAGE_SIZE - x - 1;
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

fn draw_rect(img: &mut Image, mut lower_left: (u32, u32), mut upper_right: (u32, u32), color: Color) {
    if lower_left.0 > upper_right.0 {
        mem::swap(&mut lower_left.0, &mut upper_right.0);
    }
    if lower_left.1 > upper_right.1 {
        mem::swap(&mut lower_left.1, &mut upper_right.1);
    }
    for x in lower_left.0..upper_right.0 {
        for y in lower_left.1..upper_right.1 {
            img.get_pixel_mut(x, y).0 = color;
        }
    }
}

fn draw_segment(
    img: &mut Image,
    start: (u32, u32),
    middle: (u32, u32),
    end: (u32, u32),
    width: u32,
    color: Color
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

fn colorscale(f: f64, offset: f64, saturation: f64) -> (Color, bool) {
    const PI: f64 = std::f64::consts::PI;

    let angle = START_COLOR_ANGLE + TOTAL_COLOR_ANGLE * f + offset;
    let scale = (2.0 * f - 1.0).abs() * 0.8 + 0.2; // prev 0.9, sqrt
    let rad = 0.15 * scale * saturation;
    let l = 0.85 * scale;
    let a = rad * (2.0 * PI * angle).sin();
    let b = rad * (2.0 * PI * angle).cos();
    oklab_to_srgb([l, a, b])
}
