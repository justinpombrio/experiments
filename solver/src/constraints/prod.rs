use super::{Constraint, YesNoMaybe};
use std::fmt::Debug;
use std::ops::Mul;

/// The constraint that `X1 * ... * Xn = expected` **The numbers must be non-negative!** Negative
/// numbers will lead to either the solver saying there is no answer when there is, or giving bogus
/// answers.
#[derive(Debug, Clone)]
pub struct Prod<N: Debug + Mul<Output = N> + Ord + Clone + Sized + 'static> {
    expected: N,
}

impl<N: Debug + Mul<Output = N> + Ord + Clone + Sized + 'static> Prod<N> {
    pub fn new(expected: N) -> Prod<N> {
        Prod { expected }
    }
}

impl<N: Debug + Mul<Output = N> + Ord + Clone + Sized + 'static> Constraint<N> for Prod<N> {
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

    fn check(&self, set: (N, N)) -> YesNoMaybe {
        use std::cmp::Ordering::{Equal, Greater, Less};
        use YesNoMaybe::{Maybe, No, Yes};

        match (set.0.cmp(&self.expected), self.expected.cmp(&set.1)) {
            (Equal, Equal) => Yes,
            (Less, Equal) | (Equal, Less) | (Less, Less) => Maybe,
            (Greater, _) | (_, Greater) => No,
        }
    }
}
