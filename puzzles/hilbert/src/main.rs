mod hilbert_curve;
mod oklab;

use hilbert_curve::HilbertCurve;
use image::{ImageBuffer, Rgb};
use oklab::oklab_to_srgb;
use std::mem;

const PI: f32 = std::f32::consts::PI;
const IMAGE_SIZE: u32 = 4096;
const HILBERT_SIZE: u32 = 64;
const CELL_WIDTH: u32 = IMAGE_SIZE / HILBERT_SIZE;
const LINE_WIDTH: u32 = 3 * CELL_WIDTH / 4;
const ENDPOINT_BORDER: u32 = CELL_WIDTH / 16;

const BACKGROUND_COLOR: [u8; 3] = [255, 255, 255];
const FOREGROUND_COLOR: [u8; 3] = [0, 0, 0];

type Image = ImageBuffer<Rgb<u8>, Vec<u8>>;
type Color = [u8; 3];

pub fn main() {
    let mut img = ImageBuffer::<Rgb<u8>, _>::new(IMAGE_SIZE, IMAGE_SIZE);

    // Draw background
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        pixel.0 = BACKGROUND_COLOR;
        let sx = x / CELL_WIDTH / 8;
        let sy = y / CELL_WIDTH / 8;
        if (sx + sy) % 2 == 0 {
            pixel.0 = FOREGROUND_COLOR;
        }
    }

    let scale = |point: (u32, u32)| -> (u32, u32) {
        (CELL_WIDTH * point.0 + CELL_WIDTH / 2, CELL_WIDTH * point.1 + CELL_WIDTH / 2)
    };

    // Draw curve
    let curve = HilbertCurve::new(HILBERT_SIZE);
    let mut total_count = 0;
    let mut clamped_count = 0;
    for d in 2..curve.length() {
        let start = scale(curve.dist_to_point(d - 2));
        let middle = scale(curve.dist_to_point(d - 1));
        let end = scale(curve.dist_to_point(d));
        let frac = (d as f32) / (curve.length() as f32);
        let (color, clamped) = colorscale(frac);
        total_count += 1;
        clamped_count += clamped as u32;
        draw_segment(&mut img, start, middle, end, LINE_WIDTH / 2, color);
    }
    println!("fraction clamped: {}", (clamped_count as f32) / (total_count as f32));

    // Draw start and end points
    let start_color = colorscale(0.0).0;
    let end_color = colorscale(1.0).0;
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

    img.save("puzzle.png").unwrap();
}

fn colorscale(f: f32) -> (Color, bool) {
    let angle = 4.0 * PI * f + PI;
    let scale = (2.0 * f - 1.0).abs() * 0.97 + 0.03;
    let rad = 0.132 * scale.powf(1.0 / 3.0);
    let l = 0.75 * scale.powf(1.0 / 3.0);
    let a = rad * angle.sin();
    let b = rad * angle.cos();
    oklab_to_srgb([l, a, b])
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
        let radius = (CELL_WIDTH / 2) as f32;
        let center = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
        let dmiddle = (2 * middle.0 - center.0, 2 * middle.1 - center.1);
        for x in center.0.min(dmiddle.0)..center.0.max(dmiddle.0) {
            for y in center.1.min(dmiddle.1)..center.1.max(dmiddle.1) {
                let dx = x as f32 - center.0 as f32;
                let dy = y as f32 - center.1 as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if (dist - radius).abs() < width as f32 {
                    img.get_pixel_mut(x, y).0 = color;
                }
            }
        }
    }
}
