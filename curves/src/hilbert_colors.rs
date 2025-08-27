use crate::oklab::{oklab_to_srgb, Color};

/// Given a fraction `f` between 0 and 1 along the 3D Hilbert curve, return the color when its
/// coordinates are interpreted as (r, g, b) in srgb space.
#[allow(unused)]
pub fn hilbert_color_srgb(f: f64) -> Color {
    let depth = 16usize; // 16-bit color
    let index = (f * 2usize.pow(depth as u32).pow(3) as f64).round() as usize;
    let (r, g, b) = hilbert_3d_coords(depth, index);
    [r as u16, g as u16, b as u16]
}

/// Given a fraction `f` between 0 and 1 along the 3D Hilbert curve, return the color when its
/// coordinates are interpreted as (red, green, blue) in oklab space.
pub fn hilbert_color(f: f64) -> Color {
    let f = (f + 0.0) % 1.0;
    let depth = 16usize; // 16-bit color
    let index = (f * 2usize.pow(depth as u32).pow(3) as f64).round() as usize;
    let (r, g, b) = hilbert_3d_coords(depth, index);
    let (r, g, b) = (
        r as f64 / u16::MAX as f64,
        g as f64 / u16::MAX as f64,
        b as f64 / u16::MAX as f64,
    );

    // (l, a, b)
    // = r * (1/3,  sqrt(3)/2, 1/2)
    // + g * (1/3, -sqrt(3)/2, 1/2)
    // + b * (1/3,  0,         -1)
    let s = 0.13;
    let [l, a, b] = [
        0.3 + 0.6 * (r + g + b) / 3.0,
        s * 3.0f64.sqrt() / 2.0 * (r - g),
        s * (r / 2.0 + g / 2.0 - b),
    ];
    oklab_to_srgb([l, -a, b]).unwrap_or_else(|| {
        panic!("Color out of bounds:\n  LAB = {l}, {a}, {b}\n  RGB = {r}, {g}, {b}")
    })
}

// This function is transliterated from John Burkardt's Python implementation of the same, which is
// MIT licensed.
// https://people.sc.fsu.edu/~jburkardt//py_src/hilbert_curve_3d/hilbert_curve_3d.html
/// Find the (x, y, z) position of the `index`th point along the 3D Hilbert Curve of order `depth`.
fn hilbert_3d_coords(depth: usize, index: usize) -> (usize, usize, usize) {
    let mut i = index;

    let (mut x, mut y, mut z): (usize, usize, usize) = match i % 8 {
        0 => (0, 0, 0),
        1 => (1, 0, 0),
        2 => (1, 0, 1),
        3 => (0, 0, 1),
        4 => (0, 1, 1),
        5 => (1, 1, 1),
        6 => (1, 1, 0),
        7 => (0, 1, 0),
        _ => unreachable!("i % 8 must be in range [0, 8)"),
    };
    i /= 8;

    let mut w = 2;
    while i > 0 {
        (x, y, z) = match i % 8 {
            0 => (y, z, x),
            1 => (z + w, x, y),
            2 => (z + w, x, y + w),
            3 => (w - x - 1, y, 2 * w - z - 1),
            4 => (w - x - 1, y + w, 2 * w - z - 1),
            5 => (z + w, 2 * w - x - 1, 2 * w - y - 1),
            6 => (z + w, 2 * w - x - 1, w - y - 1),
            7 => (w - y - 1, 2 * w - z - 1, x),
            _ => unreachable!("i % 8 must be in range [0, 8)"),
        };
        i /= 8;
        w *= 2; // This was wrong in the source
    }

    let max = x.max(y).max(z);
    let rmin = if max == 0 { 0 } else { max.ilog2() as usize };
    (x, y, z) = match (depth - rmin - 1) % 3 {
        0 => (x, y, z),
        1 => (y, z, x),
        2 => (z, x, y),
        _ => unreachable!("x % 3 must be in range [0, 3)"),
    };

    (x, y, z)
}

#[test]
fn test_hilbert_3d_coords() {
    let expected = [
        (0, 0, 0),
        (0, 0, 1),
        (0, 1, 1),
        (0, 1, 0),
        (1, 1, 0),
        (1, 1, 1),
        (1, 0, 1),
        (1, 0, 0),
        (2, 0, 0),
        (2, 1, 0),
        (3, 1, 0),
        (3, 0, 0),
        (3, 0, 1),
        (3, 1, 1),
        (2, 1, 1),
        (2, 0, 1),
        (2, 0, 2),
        (2, 1, 2),
        (3, 1, 2),
        (3, 0, 2),
        (3, 0, 3),
        (3, 1, 3),
        (2, 1, 3),
        (2, 0, 3),
        (1, 0, 3),
        (0, 0, 3),
        (0, 0, 2),
        (1, 0, 2),
        (1, 1, 2),
        (0, 1, 2),
        (0, 1, 3),
        (1, 1, 3),
        (1, 2, 3),
        (0, 2, 3),
        (0, 2, 2),
        (1, 2, 2),
        (1, 3, 2),
        (0, 3, 2),
        (0, 3, 3),
        (1, 3, 3),
        (2, 3, 3),
        (2, 2, 3),
        (3, 2, 3),
        (3, 3, 3),
        (3, 3, 2),
        (3, 2, 2),
        (2, 2, 2),
        (2, 3, 2),
        (2, 3, 1),
        (2, 2, 1),
        (3, 2, 1),
        (3, 3, 1),
        (3, 3, 0),
        (3, 2, 0),
        (2, 2, 0),
        (2, 3, 0),
        (1, 3, 0),
        (1, 3, 1),
        (1, 2, 1),
        (1, 2, 0),
        (0, 2, 0),
        (0, 2, 1),
        (0, 3, 1),
        (0, 3, 0),
    ];
    for (i, coords) in expected.iter().enumerate() {
        assert_eq!(hilbert_3d_coords(2, i), *coords);
    }
}

#[test]
fn test_stuff() {
    for f in [0.0, 0.1, 0.25, 0.5, 0.9] {
        let depth = 16usize;
        let index = (f * 2usize.pow(depth as u32).pow(3) as f64).round() as usize;
        let (x, y, z) = hilbert_3d_coords(depth, index);
        let n = u16::MAX;
        println!("? {depth} {index} {n} ({x}, {y}, {z})");
    }
    panic!()
}
