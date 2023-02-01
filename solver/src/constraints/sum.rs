use super::Constraint;
use std::ops::Add;

/// The constraint that `X1 + ... + Xn = expected`
pub struct Sum<N: Add<Output = N> + Ord + Sized + Clone + 'static> {
    expected: N,
}

impl<N: Add<Output = N> + Ord + Sized + Clone + 'static> Sum<N> {
    pub fn new(expected: N) -> Sum<N> {
        Sum { expected }
    }
}

impl<N: Add<Output = N> + Ord + Sized + Clone + 'static> Constraint<N> for Sum<N> {
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
