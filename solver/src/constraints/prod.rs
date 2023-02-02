use super::Constraint;
use std::ops::Mul;

/// The constraint that `X1 * ... * Xn = expected`
pub struct Prod<N: Mul<Output = N> + Ord + Clone + Sized + 'static> {
    expected: N,
}

impl<N: Mul<Output = N> + Ord + Clone + Sized + 'static> Prod<N> {
    pub fn new(expected: N) -> Prod<N> {
        Prod { expected }
    }
}

impl<N: Mul<Output = N> + Ord + Clone + Sized + 'static> Constraint<N> for Prod<N> {
    type Set = (N, N);

    const NAME: &'static str = "Prod";

    fn singleton(&self, _index: usize, elem: N) -> (N, N) {
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
