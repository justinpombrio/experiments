use std::mem;

// Algorithm from Wikipedia

pub struct HilbertCurve {
    dimension: u32,
}

impl HilbertCurve {
    /// Create a new Hilbert curve of size `dimension x dimension`.
    pub fn new(dimension: u32) -> HilbertCurve {
        // Check that dimension is a power of 2.
        assert!(
            dimension & (dimension - 1) == 0,
            "Hilbert curve dimension must be power of 2"
        );
        HilbertCurve { dimension }
    }

    /// (x, y) point -> distance along the curve.
    #[allow(unused)]
    pub fn point_to_dist(&self, point: (u32, u32)) -> u32 {
        let (mut x, mut y) = point;
        let mut d = 0;
        let mut s = self.dimension / 2;
        while s > 0 {
            let rx = ((x & s) > 0) as u32;
            let ry = ((y & s) > 0) as u32;
            d += s * s * ((3 * rx) ^ ry);
            rotate(self.dimension, &mut x, &mut y, rx, ry);
            s /= 2;
        }
        d
    }

    /// Distance along the curve -> (x, y) point.
    pub fn dist_to_point(&self, d: u32) -> (u32, u32) {
        let mut t = d;
        let (mut x, mut y) = (0, 0);
        let mut s = 1;
        while s < self.dimension {
            let rx = 1 & (t / 2);
            let ry = 1 & (t ^ rx);
            rotate(s, &mut x, &mut y, rx, ry);
            x += s * rx;
            y += s * ry;
            t /= 4;
            s *= 2;
        }
        (x, y)
    }

    /// The number of points in the curve.
    pub fn length(&self) -> u32 {
        self.dimension * self.dimension
    }
}

fn rotate(n: u32, x: &mut u32, y: &mut u32, rx: u32, ry: u32) {
    if ry == 0 {
        if rx == 1 {
            *x = n.wrapping_sub(1).wrapping_sub(*x);
            *y = n.wrapping_sub(1).wrapping_sub(*y);
        }
        mem::swap(x, y);
    }
}

#[test]
fn test_hilbert_curve() {
    for n in [1, 2, 4, 8, 16, 32] {
        let curve = HilbertCurve::new(n);
        for d in 0..n * n {
            assert_eq!(curve.point_to_dist(curve.dist_to_point(d)), d);
        }
        for x in 0..n {
            for y in 0..n {
                assert_eq!(curve.dist_to_point(curve.point_to_dist((x, y))), (x, y));
            }
        }
    }
}
