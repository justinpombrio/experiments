// TODO: temporary
#![allow(unused)]

mod color_scale;
mod curve;
//mod hilbert_curve;
mod oklab;

use color_scale::{color_scale, ColorScaleConfig};
use curve::HILBERT_CURVE;
//use hilbert_curve::HilbertCurve;
use image::{ImageBuffer, Rgb};
//use oklab::oklab_to_srgb;
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
const BORDER_SIZE: u32 = 0;
const HILBERT_ITERS: u32 = 4;
const HILBERT_SIZE: u32 = 2_u32.pow(HILBERT_ITERS);
//const HILBERT_CURVE_LEN: u32 = HILBERT_SIZE * HILBERT_SIZE;
const CELL_WIDTH: u32 = IMAGE_SIZE / HILBERT_SIZE;

// Squares
const DRAW_SQUARES: bool = false;

// Curve
const DRAW_CURVE: bool = true;
const LINE_WIDTH: u32 = 7 * CELL_WIDTH / 8;
const USE_FIXED_CURVE_COLOR: bool = false;
const CURVE_COLOR: [u16; 3] = [30 * 256, 30 * 256, 30 * 256];
const ENDPOINT_BORDER: u32 = CELL_WIDTH / 16;

// Checkerboard
const DRAW_CHECKERBOARD: bool = false;
const FOREGROUND_COLOR: [u16; 3] = [u16::MAX, u16::MAX, u16::MAX];
const BACKGROUND_COLOR: [u16; 3] = [0, 0, 0];

type Image = ImageBuffer<Rgb<u16>, Vec<u16>>;
type Color = [u16; 3];

fn main() {
    let mut canvas = Canvas::new((250, 250));
    canvas.draw_checkerboard(
        Bounds {
            min_x: 5,
            max_x: 245,
            min_y: 5,
            max_y: 245,
        },
        (3, 3),
        [250 * 256, 20 * 256, 20 * 256],
        [20 * 256, 250 * 256, 20 * 256],
    );
    canvas.save();
}

pub fn main_old() {
    let mut img =
        ImageBuffer::<Rgb<u16>, _>::new(IMAGE_SIZE + 2 * BORDER_SIZE, IMAGE_SIZE + 2 * BORDER_SIZE);

    // Draw background
    for pixel in img.pixels_mut() {
        pixel.0 = BORDER_COLOR;
    }
    if DRAW_CHECKERBOARD {
        println!("draw checkerboard");
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
        println!("draw squares");
        let curve = HILBERT_CURVE.expand(HILBERT_ITERS as usize);
        let len = curve.len();
        for (d, point) in curve.enumerate() {
            let point = scale(point);
            let frac = (d as f64) / (len as f64);
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
        let len = curve.len();
        let mut start = scale(curve.next().unwrap());
        let mut middle = scale(curve.next().unwrap());
        for (d, point) in curve.enumerate() {
            let end = scale(point);
            let frac = (d as f64) / (len as f64);
            let color = if USE_FIXED_CURVE_COLOR {
                CURVE_COLOR
            } else {
                color_scale(frac, COLORS).0
            };
            draw_segment(&mut img, start, middle, end, LINE_WIDTH / 2, color);
            start = middle;
            middle = end;
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
    }

    println!("Saving image to hilbert.png");
    img.save("hilbert.png").unwrap();
}

struct Canvas {
    image: Image,
}

#[derive(Debug, Clone, Copy)]
struct Bounds<N: Copy> {
    min_x: N,
    min_y: N,
    max_x: N,
    max_y: N,
}

impl Canvas {
    fn new((width, height): (u32, u32)) -> Canvas {
        Canvas {
            image: ImageBuffer::<Rgb<u16>, _>::new(width, height),
        }
    }

    fn save(&mut self) {
        self.image.save("curve.png").unwrap();
    }

    fn draw_checkerboard(
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

    /*
    fn draw_curve(&mut self,
                  curve: impl ExactSizeIterator<Item = (f64, f64),
                  image_bounds: Bounds<u32>,
                  curve_bounds: Bounds<f64>,
                  colors: |f64| -> (Color, bool)) {
        let transform = |point: (f64, f64)| -> (u32, u32) {
            ???
            CELL_WIDTH * point.0.round() as u32 + CELL_WIDTH / 2,
            CELL_WIDTH * point.1.round() as u32 + CELL_WIDTH / 2,
        }

        let len = curve.len();
        let mut start = transform(curve.next().unwrap());
        let mut middle = transform(curve.next().unwrap());
        for (d, point) in curve.enumerate() {
            let end = transform(point);
            let frac = (d as f64) / (len as f64);
            let color = if USE_FIXED_CURVE_COLOR {
                CURVE_COLOR
            } else {
                color_scale(frac, COLORS).0
            };
            draw_segment(&mut img, start, middle, end, LINE_WIDTH / 2, color);
            start = middle;
            middle = end;
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
    }
    */

    // Draw the segment of the curve located at `middle`, between `start` and `end`. Requires:
    // - len(start->middle) = len(middle->end)
    // - start->middle is colinear with or at a right angle to middle->end.
    fn draw_segment(
        &mut self,
        start: (u32, u32),
        middle: (u32, u32),
        end: (u32, u32),
        width: u32,
        color: Color,
    ) {
        if start.0 == end.0 {
            self.draw_rect(
                Bounds {
                    min_x: start.0 - width,
                    max_x: start.0 + width,
                    min_y: (start.1 + middle.1) / 2,
                    max_y: (middle.1 + end.1) / 2,
                },
                color,
            );
        } else if start.1 == end.1 {
            self.draw_rect(
                Bounds {
                    min_x: (start.0 + middle.0) / 2,
                    max_x: (middle.0 + end.0) / 2,
                    min_y: start.1 - width,
                    max_y: start.1 + width,
                },
                color,
            );
        } else {
            let radius = ((start.0 as i32 - middle.0 as i32).abs()
                + (start.1 as i32 - middle.1 as i32).abs()) as f64
                / 2.0;
            let center = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
            let dmiddle = (2 * middle.0 - center.0, 2 * middle.1 - center.1);
            for x in center.0.min(dmiddle.0)..center.0.max(dmiddle.0) {
                for y in center.1.min(dmiddle.1)..center.1.max(dmiddle.1) {
                    let dx = x as f64 - center.0 as f64;
                    let dy = y as f64 - center.1 as f64;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if (dist - radius).abs() < width as f64 {
                        self.image.get_pixel_mut(x, y).0 = color;
                    }
                }
            }
        }
    }

    fn fill(&mut self, color: Color) {
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

    fn draw_rect(&mut self, mut bounds: Bounds<u32>, color: Color) {
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

    fn draw_arc(&mut self, center: (u32, u32), corner: (u32, u32), color: Color) {
        fn diff(a: u32, b: u32) -> u32 {
            a.max(b) - a.min(b)
        }
        let (cx, cy) = center;
        let (ax, ay) = corner;
        let (rx, ry) = (diff(cx, ax), diff(cy, ay));
        let (min_x, max_x) = (cx.min(ax), cx.max(ax));
        let (min_y, max_y) = (cy.min(ay), cy.max(ay));
        for x in min_x..max_x {
            for y in min_y..max_y {
                let (dx, dy) = (diff(x, cx), diff(y, cy));
                // Equivalent to the ellipse equation (dx/rx)^2 + (dy/rx)^2 <= 1.0
                if dx * dx / rx * ry + dy * dy / ry * rx <= rx * ry {
                    self.image.get_pixel_mut(x, y).0 = color;
                }
            }
        }
    }
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
        let radius = ((start.0 as i32 - middle.0 as i32).abs()
            + (start.1 as i32 - middle.1 as i32).abs()) as f64
            / 2.0;
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
