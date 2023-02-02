use super::Constraint;
use std::fmt::Debug;
use std::marker::PhantomData;

/// The constraint that `pred(X1, ..., Xn)` holds.
pub struct Pred<const N: usize, T: Debug + Clone + Sized + 'static> {
    pred: Box<dyn Fn(&[T; N]) -> bool>,
    _phantom: PhantomData<T>,
}

impl<const N: usize, T: Debug + Clone + Sized + 'static> Pred<N, T> {
    pub fn new(pred: impl Fn(&[T; N]) -> bool + 'static) -> Pred<N, T> {
        Pred {
            pred: Box::new(pred),
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, T: Debug + Clone + Sized + 'static> Constraint<T> for Pred<N, T> {
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
            if elem.is_none() {
                result[i] = None;
            }
        }
        result
    }

    fn check(&self, set: Self::Set) -> bool {
        if let Some(set) = unwrap_array(set) {
            (self.pred)(&set)
        } else {
            true
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
