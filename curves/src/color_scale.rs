use crate::arith::{interpolate, Point};
use crate::hilbert_3d::hilbert_3d_coords;
use crate::oklab::{oklab_hsl_to_srgb, oklab_to_srgb};

pub use crate::srgb::Color;

/// The sawtooth function which starts at 0.0, progresses linearly to 1.0 at f=0.5, returns to 0.0
/// at f=1.0, and cycles indefinitely.
pub fn sawtooth(f: f64) -> f64 {
    1.0 - (2.0 * (f % 1.0) - 1.0).abs()
}

/// As `f` scales from 0.0 to 1.0, the result scales linearly from `start` to `end`.
pub fn scale(f: f64, min: f64, max: f64) -> f64 {
    min + f * (max - min)
}

pub fn orbit(
    f: f64,
    (big_start, big_end, big_rad): (f64, f64, f64),
    (little_start, little_end, little_rad): (f64, f64, f64),
) -> (f64, f64) {
    let big_vec = Point::cis(interpolate(f, big_start, big_end)) * big_rad;
    let little_vec = Point::cis(interpolate(f, little_start, little_end)) * little_rad;
    let vector = big_vec + little_vec;
    (vector.abs(), vector.angle())
}

/// Convert an OKLAB hue,sat,lit color into an SRGB color.
pub fn hsl(oklab_hsl: [f64; 3]) -> Color {
    match oklab_hsl_to_srgb(oklab_hsl) {
        Some(color) => color,
        None => panic!("Color out of bounds: {:?}", oklab_hsl),
    }
}

/// Convert an OKLAB r,g,b color into an SRGB color. The output is scaled so that the r,g,b values
/// may range from 0.0 to 1.0 while producing realizable colors.
pub fn rgb([r, g, b]: [f64; 3]) -> Color {
    // (l, a, b)
    // = r * (1/3,  sqrt(3)/2, 1/2)
    // + g * (1/3, -sqrt(3)/2, 1/2)
    // + b * (1/3,  0,         -1)

    let max_sat = 0.141;
    let min_lit = 0.3;
    let [l, a, b] = [
        min_lit + (1.0 - min_lit) * (r + g + b) / 3.0,
        max_sat * 3.0f64.sqrt() / 2.0 * (r - g),
        max_sat * (r / 2.0 + g / 2.0 - b),
    ];
    oklab_to_srgb([l, a, b]).unwrap_or_else(|| {
        panic!("Color out of bounds:\n  LAB = {l}, {a}, {b}\n  RGB = {r}, {g}, {b}")
    })
}

/// Given a fraction `f` between 0 and 1 along the 3D Hilbert curve, return the color when its
/// coordinates are interpreted as 16-bit (r, g, b).
fn hilbert_color_rgb(f: f64) -> [u16; 3] {
    let depth = 16usize; // 16-bit color
    let index = (f * 2usize.pow(depth as u32).pow(3) as f64).round() as usize;
    let (r, g, b) = hilbert_3d_coords(depth, index);
    [r as u16, g as u16, b as u16]
}

/// Given a fraction `f` between 0 and 1 along the 3D Hilbert curve, return the color when its
/// coordinates are interpreted as (red, green, blue) in oklab space.
pub fn hilbert_color(f: f64) -> Color {
    let [r16, g16, b16] = hilbert_color_rgb(f);
    let rgb1 = [
        r16 as f64 / u16::MAX as f64,
        g16 as f64 / u16::MAX as f64,
        b16 as f64 / u16::MAX as f64,
    ];
    rgb(rgb1)
}

pub fn color_scale_from_data<const N: usize>(f: f64, data: [[f64; 3]; N]) -> Color {
    let g = f * (N - 1) as f64;
    let (i, m) = if f == 1.0 {
        (N - 2, 1.0)
    } else {
        (g as usize, g % 1.0)
    };
    let [r0, g0, b0] = data[i];
    let [r1, g1, b1] = data[i + 1];
    let [r, g, b] = [
        interpolate(m, r0, r1),
        interpolate(m, g0, g1),
        interpolate(m, b0, b1),
    ];
    Color([
        (r * u16::MAX as f64) as u16,
        (g * u16::MAX as f64) as u16,
        (b * u16::MAX as f64) as u16,
    ])
}
