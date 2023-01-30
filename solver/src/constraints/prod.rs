use super::Constraint;
use std::ops::Mul;

/// Numbers that can be multiplied (plus other conveniences)
pub trait Mullable: Mul<Self, Output = Self> + Ord + Clone + Sized + 'static {}

/// The constraint that `X1 * ... * Xn = expected`
pub struct Prod<N: Mullable> {
    expected: N,
}

impl<N: Mullable> Prod<N> {
    pub fn new(expected: N) -> Prod<N> {
        Prod { expected }
    }
}

impl<N: Mullable> Constraint<N> for Prod<N> {
    type Set = (N, N);

    const NAME: &'static str = "Sum";

    fn singleton(&self, elem: N) -> (N, N) {
        (elem.clone(), elem)
    }

    fn and(&self, a: (N, N), b: (N, N)) -> (N, N) {
        (a.0 * b.0, a.1 * b.1)
    }

    fn or(&self, a: (N, N), b: (N, N)) -> (N, N) {
        (a.0.min(b.0), a.1.max(b.1))
    }

    fn check(&self, set: (N, N)) -> bool {
        set.0 <= self.expected.clone() && self.expected.clone() <= set.1
    }
}

macro_rules! define_prod {
    ($ty:ident) => {
        impl Mullable for $ty {}
    };
}

define_prod!(u8);
define_prod!(u16);
define_prod!(u32);
define_prod!(u64);
define_prod!(u128);
