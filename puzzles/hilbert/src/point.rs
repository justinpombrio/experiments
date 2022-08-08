use std::ops::{Add, Sub, Mul};

trait Number : Clone + Copy + Add<Output=Self> + Sub<Output=Self> + Mul<Output = Self> {}

impl Number for f64 {}
impl Number for u32 {}

struct Point<N: Number>
{
    x: N,
    y: N,
}

impl<N: Number> Mul<N> for Point<N> {
    type Output = Point<N>;

    fn mul(self, scalar: N) -> Point<N> {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl<N: Number> Add<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn add(self, other: Point<N>) -> Point<N> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<N: Number> Sub<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn sub(self, other: Point<N>) -> Point<N> {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
