use super::{Constraint, YesNoMaybe};
use std::fmt::Debug;
use std::marker::PhantomData;

/// The constraint that `pred(X1, ..., Xn)` holds.
pub struct Pred<const N: usize, T: Debug + PartialEq + Clone + Sized + 'static> {
    pred: Box<dyn Fn(&[T; N]) -> bool>,
    _phantom: PhantomData<T>,
}

impl<const N: usize, T: Debug + PartialEq + Clone + Sized + 'static> Pred<N, T> {
    pub fn new(pred: impl Fn(&[T; N]) -> bool + 'static) -> Pred<N, T> {
        Pred {
            pred: Box::new(pred),
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, T: Debug + PartialEq + Clone + Sized + 'static> Constraint<T> for Pred<N, T> {
    const NAME: &'static str = "Pred";

    type Set = [Option<T>; N];

    fn singleton(&self, index: usize, elem: T) -> Self::Set {
        let mut result = std::array::from_fn(|_| None);
        result[index] = Some(elem);
        result
    }

    fn and(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let mut result = a;
        for (i, elem) in b.into_iter().enumerate() {
            if let Some(elem) = elem {
                result[i] = Some(elem);
            }
        }
        result
    }

    fn or(&self, a: Self::Set, b: Self::Set) -> Self::Set {
        let mut result = a;
        for (i, elem) in b.into_iter().enumerate() {
            if &result[i] != &elem {
                result[i] = None;
            }
        }
        result
    }

    fn check(&self, set: Self::Set) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        if let Some(set) = unwrap_array(set) {
            if (self.pred)(&set) {
                Yes
            } else {
                No
            }
        } else {
            Maybe
        }
    }
}

fn unwrap_array<const N: usize, T: Clone>(opt_array: [Option<T>; N]) -> Option<[T; N]> {
    for elem in &opt_array {
        if elem.is_none() {
            return None;
        }
    }
    Some(std::array::from_fn(|i| opt_array[i].clone().unwrap()))
}

#[test]
fn test_sum() {
    use YesNoMaybe::{Maybe, No, Yes};

    let s = Pred::new(|[a, b]| a < b);

    assert_eq!(s.singleton(0, 10), [Some(10), None]);
    assert_eq!(s.singleton(1, 10), [None, Some(10)]);
    assert_eq!(s.or(s.singleton(0, 10), s.singleton(0, 20)), [None, None]);
    assert_eq!(
        s.and(s.singleton(0, 10), s.singleton(1, 20)),
        [Some(10), Some(20)]
    );

    assert_eq!(s.check([None, None]), Maybe);
    assert_eq!(s.check([Some(1), None]), Maybe);
    assert_eq!(s.check([None, Some(1)]), Maybe);
    assert_eq!(s.check([Some(1), Some(2)]), Yes);
    assert_eq!(s.check([Some(2), Some(2)]), No);
}
