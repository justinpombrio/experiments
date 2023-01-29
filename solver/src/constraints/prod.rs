use super::Constraint;
use crate::state::State;
use std::ops::{Div, Mul};

pub trait Mullable: Mul<Self, Output = Self> + Div<Self, Output = Self> + Ord + Sized {
    fn one() -> Self;
}

pub struct Prod<S: State>
where
    S::Value: Mullable,
{
    expected: S::Value,
    params: Vec<S::Var>,
    map: Box<dyn Fn(usize, S::Value) -> (S::Value, S::Value)>,
}

impl<S: State> Prod<S>
where
    S::Value: Mullable,
{
    pub fn new(params: impl IntoIterator<Item = S::Var>, expected: S::Value) -> Prod<S> {
        Prod::generic(params, expected, |_, n| n)
    }

    pub fn generic(
        params: impl IntoIterator<Item = S::Var>,
        expected: S::Value,
        map: impl Fn(usize, S::Value) -> S::Value + 'static,
    ) -> Prod<S> {
        Prod::generic_range(params, expected, move |i, n| {
            let n2 = map(i, n);
            (n2.clone(), n2)
        })
    }

    pub fn generic_range(
        params: impl IntoIterator<Item = S::Var>,
        expected: S::Value,
        map: impl Fn(usize, S::Value) -> (S::Value, S::Value) + 'static,
    ) -> Prod<S> {
        Prod {
            params: params.into_iter().collect::<Vec<_>>(),
            expected,
            map: Box::new(map),
        }
    }
}

impl<S: State> Constraint<S> for Prod<S>
where
    S::Value: Mullable,
{
    type Set = (S::Value, S::Value);

    const NAME: &'static str = "Sum";

    fn new_set(&self, index: usize, elem: S::Value) -> (S::Value, S::Value) {
        (self.map)(index, elem)
    }

    fn none(&self) -> (S::Value, S::Value) {
        (Mullable::one(), Mullable::one())
    }

    fn and(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (a.0 * b.0, a.1 * b.1)
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

macro_rules! define_prod {
    ($ty:ident) => {
        impl Mullable for $ty {
            fn one() -> $ty {
                1
            }
        }
    };
}

define_prod!(u8);
define_prod!(u16);
define_prod!(u32);
define_prod!(u64);
define_prod!(u128);
