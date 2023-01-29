use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

pub trait Arith: fmt::Debug + Clone + 'static {
    fn identity() -> Self;

    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

macro_rules! define_arith {
    ($ty:ident) => {
        impl Arith for $ty {
            fn identity() -> $ty {
                0
            }

            fn add(self, other: $ty) -> $ty {
                self + other
            }
            fn sub(self, other: $ty) -> $ty {
                self - other
            }
            fn min(self, other: $ty) -> $ty {
                Ord::min(self, other)
            }
            fn max(self, other: $ty) -> $ty {
                Ord::max(self, other)
            }
        }
    };
}

define_arith!(u8);
define_arith!(u16);
define_arith!(u32);
define_arith!(u64);
define_arith!(u128);

define_arith!(i8);
define_arith!(i16);
define_arith!(i32);
define_arith!(i64);
define_arith!(i128);

impl Arith for f32 {
    fn identity() -> f32 {
        0.0
    }

    fn add(self, other: f32) -> f32 {
        self + other
    }
    fn sub(self, other: f32) -> f32 {
        self - other
    }
    fn min(self, other: f32) -> f32 {
        f32::min(self, other)
    }
    fn max(self, other: f32) -> f32 {
        f32::max(self, other)
    }
}

impl Arith for f64 {
    fn identity() -> f64 {
        0.0
    }

    fn add(self, other: f64) -> f64 {
        self + other
    }
    fn sub(self, other: f64) -> f64 {
        self - other
    }
    fn min(self, other: f64) -> f64 {
        f64::min(self, other)
    }
    fn max(self, other: f64) -> f64 {
        f64::max(self, other)
    }
}

#[derive(Debug, Clone)]
struct Multiset<V: fmt::Debug + Hash + Eq + Clone + 'static>(HashMap<V, u32>);

impl<V: fmt::Debug + Hash + Eq + Clone + 'static> Multiset<V> {
    fn merge(self, other: Multiset<V>, f: impl Fn(u32, u32) -> u32) -> Multiset<V> {
        let mut result = self;
        for (key, count) in other.0 {
            *result.0.get_mut(&key).unwrap() = f(result.0[&key], count);
        }
        result
    }
}

impl<V: fmt::Debug + Hash + Eq + Clone + 'static> Arith for Multiset<V> {
    fn identity() -> Multiset<V> {
        Multiset(HashMap::new())
    }

    fn add(self, other: Multiset<V>) -> Multiset<V> {
        self.merge(other, |c1, c2| c1 + c2)
    }

    fn sub(self, other: Multiset<V>) -> Multiset<V> {
        self.merge(other, |c1, c2| c1 - c2)
    }

    fn min(self, other: Multiset<V>) -> Multiset<V> {
        self.merge(other, |c1, c2| Arith::min(c1, c2))
    }

    fn max(self, other: Multiset<V>) -> Multiset<V> {
        self.merge(other, |c1, c2| Arith::max(c1, c2))
    }
}
