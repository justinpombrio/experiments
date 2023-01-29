use super::Constraint;
use crate::state::State;

pub trait Summable {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn leq(self, other: Self) -> bool;
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

    pub fn new_linear(
        params: impl IntoIterator<Item = S::Var>,
        expected: S::Value,
        coefficients: impl IntoIterator<Item = S::Value>,
    ) -> Sum<S> {
        let coefficients = coefficients.into_iter().collect::<Vec<_>>();
        Sum::new_generic(params, expected, move |i, n| {
            S::Value::mul(coefficients[i].clone(), n)
        })
    }

    pub fn new_generic(
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

    pub fn new_generic_range(
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
        (Summable::add(a.0, b.0), Summable::add(a.1, b.1))
    }

    fn andnot(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (Summable::sub(a.0, b.0), Summable::sub(a.1, b.1))
    }

    fn or(&self, a: (S::Value, S::Value), b: (S::Value, S::Value)) -> (S::Value, S::Value) {
        (Summable::min(a.0, b.0), Summable::max(a.1, b.1))
    }

    fn params(&self) -> &[S::Var] {
        &self.params
    }

    fn check(&self, set: (S::Value, S::Value)) -> bool {
        Summable::leq(set.0, self.expected.clone()) && Summable::leq(self.expected.clone(), set.1)
    }
}

macro_rules! define_sum {
    ($ty:ident) => {
        impl Summable for $ty {
            fn zero() -> $ty {
                0
            }

            fn add(self, other: $ty) -> $ty {
                self + other
            }

            fn sub(self, other: $ty) -> $ty {
                self - other
            }

            fn mul(self, other: $ty) -> $ty {
                self * other
            }

            fn min(self, other: $ty) -> $ty {
                std::cmp::min(self, other)
            }

            fn max(self, other: $ty) -> $ty {
                std::cmp::max(self, other)
            }

            fn leq(self, other: $ty) -> bool {
                self <= other
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
