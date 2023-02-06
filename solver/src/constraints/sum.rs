use super::{Constraint, YesNoMaybe};
use std::fmt::Debug;
use std::ops::Add;

/// The constraint that `X1 + ... + Xn = expected`
#[derive(Debug, Clone)]
pub struct Sum<N: Debug + Add<Output = N> + Ord + Sized + Clone + 'static> {
    expected: N,
}

impl<N: Debug + Add<Output = N> + Ord + Sized + Clone + 'static> Sum<N> {
    pub fn new(expected: N) -> Sum<N> {
        Sum { expected }
    }
}

impl<N: Debug + Add<Output = N> + Ord + Sized + Clone + 'static> Constraint<N> for Sum<N> {
    type Set = (N, N);

    const NAME: &'static str = "Sum";

    fn singleton(&self, _index: usize, elem: N) -> (N, N) {
        (elem.clone(), elem)
    }

    fn and(&self, a: (N, N), b: (N, N)) -> (N, N) {
        (a.0 + b.0, a.1 + b.1)
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

#[test]
fn test_sum() {
    use YesNoMaybe::{Maybe, No, Yes};

    let s = Sum::new(3);

    let one = || s.singleton(0, 1);
    let two = || s.singleton(0, 2);
    let three = || s.singleton(0, 3);

    assert_eq!(s.and(one(), two()), three());
    assert_eq!(s.or(one(), three()), (1, 3));
    assert_eq!(s.and(s.or(one(), two()), s.or(one(), three())), (2, 5));

    assert_eq!(s.check((1, 2)), No);
    assert_eq!(s.check((4, 4)), No);
    assert_eq!(s.check((2, 3)), Maybe);
    assert_eq!(s.check((1, 5)), Maybe);
    assert_eq!(s.check((3, 3)), Yes);
}
