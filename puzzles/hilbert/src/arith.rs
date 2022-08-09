use std::ops::{Add, Div, Mul, Sub};

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

pub fn interpolate<N: Number>(f: f64, start: N, end: N) -> N
where
    N: Mul<f64, Output = N>,
{
    start + (end - start) * f
}

impl Point<f64> {
    pub fn dist(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
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
