use graphics::math::identity;
use graphics::Transformed;
use std::f64::consts::PI;
use std::ops::Mul;
use vecmath::mat2x3_inv;

#[derive(Clone, Copy, Debug)]
pub struct Transform([[f64; 3]; 2]);

impl Transform {
    pub fn id() -> Transform {
        Transform(identity())
    }

    pub fn from_matrix(matrix: [[f64; 3]; 2]) -> Transform {
        Transform(matrix)
    }

    pub fn to_matrix(self) -> [[f64; 3]; 2] {
        self.0
    }

    pub fn trans(self, x: f64, y: f64) -> Transform {
        Transform(self.0.trans(x, y))
    }

    pub fn trans_by(self, [x, y]: [f64; 2]) -> Transform {
        self.trans(x, y)
    }

    #[allow(unused)]
    pub fn trans_neg(self, x: f64, y: f64) -> Transform {
        self.trans(-x, -y)
    }

    pub fn trans_by_neg(self, [x, y]: [f64; 2]) -> Transform {
        self.trans(-x, -y)
    }

    #[allow(unused)]
    pub fn rotate(self, turns: f64) -> Transform {
        Transform(self.0.rot_rad(turns / 2.0 / PI))
    }

    pub fn scale(self, x: f64, y: f64) -> Transform {
        Transform(self.0.scale(x, y))
    }

    pub fn scale_by(self, [x, y]: [f64; 2]) -> Transform {
        self.scale(x, y)
    }

    pub fn scale_inv(self, x: f64, y: f64) -> Transform {
        Transform(self.0.scale(1.0 / x, 1.0 / y))
    }

    pub fn scale_by_inv(self, [x, y]: [f64; 2]) -> Transform {
        Transform(self.0.scale(1.0 / x, 1.0 / y))
    }

    pub fn zoom(self, u: f64) -> Transform {
        self.scale(u, u)
    }

    pub fn flip_vert(self) -> Transform {
        self.scale(1.0, -1.0)
    }

    #[allow(unused)]
    pub fn flip_horz(self) -> Transform {
        self.scale(-1.0, 1.0)
    }

    pub fn centered(self) -> Transform {
        self.trans(-0.5, -0.5)
    }

    pub fn shear(self, x: f64, y: f64) -> Transform {
        Transform(self.0.shear(x, y))
    }

    pub fn then(self, other: Transform) -> Transform {
        Transform(self.0.append_transform(other.0))
    }

    #[allow(unused)]
    pub fn shear_by(self, [x, y]: [f64; 2]) -> Transform {
        self.shear(x, y)
    }

    #[allow(unused)]
    pub fn inverse(self) -> Transform {
        Transform(mat2x3_inv(self.0))
    }

    #[allow(unused)]
    pub fn point(self) -> [f64; 2] {
        let [[_, _, x], [_, _, y]] = self.0;
        [x, y]
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self: Transform, other: Transform) -> Transform {
        self.then(other)
    }
}

impl Mul<[f64; 2]> for Transform {
    type Output = [f64; 2];

    fn mul(self: Transform, [x, y]: [f64; 2]) -> [f64; 2] {
        let [[a, b, c], [d, e, f]] = self.0;
        [a * x + b * y + c, d * x + e * y + f]
    }
}

#[cfg(test)]
mod test_transform {
    use super::Transform;

    #[test]
    fn test_transform() {
        let id = Transform::id();

        assert_eq!(
            id.trans(1.0, 0.0)
                .zoom(2.0)
                .trans(0.0, 1.0)
                .inverse()
                .point(),
            [-0.5, -1.0]
        );

        let a = id.trans(0.0, 1.0).scale(2.0, 1.0);
        let b = id.trans(1.0, 0.0).scale(1.0, 3.0);
        assert_eq!((a * b).trans(10.0, 10.0).point(), [22.0, 31.0]);
        assert_eq!((a * b.trans(10.0, 10.0)).point(), [22.0, 31.0]);
    }
}
