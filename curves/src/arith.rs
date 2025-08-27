use std::ops::{Add, Div, Mul, Sub};

const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

pub trait Number:
    std::fmt::Debug
    + Clone
    + Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds<N: Number> {
    pub min: Point<N>,
    pub max: Point<N>,
}

#[derive(Debug, Clone, Copy)]
pub struct Point<N: Number> {
    pub x: N,
    pub y: N,
}

impl Number for f64 {}
impl Number for u32 {}
impl<N: Number> Number for Point<N> {}

pub fn interpolate<N>(f: f64, start: N, end: N) -> N
where
    N: Number + Mul<f64, Output = N>,
{
    start + (end - start) * f
}

impl Point<f64> {
    pub fn zero() -> Point<f64> {
        Point { x: 0.0, y: 0.0 }
    }

    pub fn cis(turns: f64) -> Point<f64> {
        let angle_rad = turns * RADS_PER_TURN;
        Point {
            x: angle_rad.cos(),
            y: angle_rad.sin(),
        }
    }

    pub fn abs(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn angle(self) -> f64 {
        self.y.atan2(self.x) / RADS_PER_TURN
    }
}

impl Point<f64> {
    pub fn rotate_quarter_turn(self) -> Point<f64> {
        Point {
            x: self.y,
            y: -self.x,
        }
    }
}

impl<N: Number> Add<N> for Point<N> {
    type Output = Point<N>;

    fn add(self, scalar: N) -> Point<N> {
        Point {
            x: self.x + scalar,
            y: self.y + scalar,
        }
    }
}

impl<N: Number> Sub<N> for Point<N> {
    type Output = Point<N>;

    fn sub(self, scalar: N) -> Point<N> {
        Point {
            x: self.x - scalar,
            y: self.y - scalar,
        }
    }
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

impl<N: Number> Div<N> for Point<N> {
    type Output = Point<N>;

    fn div(self, scalar: N) -> Point<N> {
        Point {
            x: self.x / scalar,
            y: self.y / scalar,
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

impl<N: Number> Mul<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn mul(self, other: Point<N>) -> Point<N> {
        Point {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }
}

impl<N: Number> Div<Point<N>> for Point<N> {
    type Output = Point<N>;

    fn div(self, other: Point<N>) -> Point<N> {
        Point {
            x: self.x / other.x,
            y: self.y / other.y,
        }
    }
}
