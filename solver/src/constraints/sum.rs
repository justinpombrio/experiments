use super::Constraint;
use crate::state::State;
use std::ops::{Add, Mul, Sub};

pub trait Summable:
    Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<Self, Output = Self> + Ord + Sized
{
    fn zero() -> Self;
}

pub struct Sum<S: State>
where
    S::Value: Summable,
{
    expected: S::Value,
    params: Vec<S::Var>,
    map: Box<dyn Fn(usize, S::Value) -> (S::Value, S::Value)>,
}

impl<S: State> Sum<S>
where
    S::Value: Summable,
{
    pub fn new(params: impl IntoIterator<Item = S::Var>, expected: S::Value) -> Sum<S> {
        Sum {
            params: params.into_iter().collect::<Vec<_>>(),
            expected,
            map: Box::new(|_, n| (n.clone(), n)),
        }
    }

    pub fn linear(
        coefficients_and_params: impl IntoIterator<Item = (S::Value, S::Var)>,
        expected: S::Value,
    ) -> Sum<S> {
        let (coefficients, params) = coefficients_and_params
            .into_iter()
            .unzip::<_, _, Vec<_>, Vec<_>>();
        Sum::generic(params, expected, move |i, n| coefficients[i].clone() * n)
    }

    pub fn generic(
        params: impl IntoIterator<Item = S::Var>,
        expected: S::Value,
        map: impl Fn(usize, S::Value) -> S::Value + 'static,
    ) -> Sum<S> {
        Sum {
            params: params.into_iter().collect::<Vec<_>>(),
            expected,
            map: Box::new(move |i, n| {
                let n2 = map(i, n);
                (n2.clone(), n2)
            }),
        }
    }

    pub fn generic_range(
        params: impl IntoIterator<Item = S::Var>,
        expected: S::Value,
        map: impl Fn(usize, S::Value) -> (S::Value, S::Value) + 'static,
    ) -> Sum<S> {
        Sum {
            params: params.into_iter().collect::<Vec<_>>(),
            expected,
            map: Box::new(map),
        }
    }
}

impl<S: State> Constraint<S> for Sum<S>
where
    S::Value: Summable,
{
    type Set = (S::Value, S::Value);

    const NAME: &'static str = "Sum";

    fn new_set(&self, index: usize, elem: S::Value) -> (S::Value, S::Value) {
        (self.map)(index, elem)
    }

    fn none(&self) -> (S::Value, S::Value) {
        (Summable::zero(), Summable::zero())
    }

    fn and(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (a.0 + b.0, a.1 + b.1)
    }

    fn andnot(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (a.0 - b.0, a.1 - b.1)
    }

    fn or(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (a.0.min(b.0), a.1.max(b.1))
    }

    fn params(&self) -> &[S::Var] {
        &self.params
    }

    fn check(&self, set: (S::Value, S::Value)) -> bool {
        set.0 <= self.expected.clone() && self.expected.clone() <= set.1
    }
}

macro_rules! define_sum {
    ($ty:ident) => {
        impl Summable for $ty {
            fn zero() -> $ty {
                0
            }
        }
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
