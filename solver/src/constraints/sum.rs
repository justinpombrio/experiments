use super::Constraint;
use std::ops::{Add, Mul};

/// Numbers that can be summed (plus other conveniences)
pub trait Summable:
    Add<Self, Output = Self> + Mul<Self, Output = Self> + Ord + Sized + Clone + 'static
{
}

/// The constraint that `X1 + ... + Xn = expected`
pub struct Sum<N: Summable> {
    expected: N,
}

impl<N: Summable> Sum<N> {
    pub fn new(expected: N) -> Sum<N> {
        Sum { expected }
    }
}

impl<N: Summable> Constraint<N> for Sum<N> {
    type Set = (N, N);

    const NAME: &'static str = "Sum";

    fn singleton(&self, elem: N) -> (N, N) {
        (elem.clone(), elem)
    }

    fn and(&self, a: (N, N), b: (N, N)) -> (N, N) {
        (a.0 + b.0, a.1 + b.1)
    }

    fn or(&self, a: (N, N), b: (N, N)) -> (N, N) {
        (a.0.min(b.0), a.1.max(b.1))
    }

    fn check(&self, set: (N, N)) -> bool {
        set.0 <= self.expected.clone() && self.expected.clone() <= set.1
    }
}

macro_rules! define_sum {
    ($ty:ident) => {
        impl Summable for $ty {}
    };
}

define_sum!(u8);
define_sum!(u16);
define_sum!(u32);
define_sum!(u64);
define_sum!(u128);

define_sum!(i8);
define_sum!(i16);
define_sum!(i32);
define_sum!(i64);
define_sum!(i128);
