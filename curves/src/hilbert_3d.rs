// This function is transliterated from John Burkardt's Python implementation of the same, which is
// MIT licensed.
// https://people.sc.fsu.edu/~jburkardt//py_src/hilbert_curve_3d/hilbert_curve_3d.html
/// Find the (x, y, z) position of the `index`th point along the 3D Hilbert Curve of order `depth`.
pub fn hilbert_3d_coords(depth: usize, index: usize) -> (usize, usize, usize) {
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
