use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;

pub trait Property: Clone + 'static {
    fn any() -> Self;

    fn and(self, other: Self) -> Self;
    fn andnot(self, other: Self) -> Self;
    fn or(self, other: Self) -> Self;
}

pub trait Summable: fmt::Debug + Clone + 'static {
    fn zero() -> Self;

    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
}

macro_rules! define_summable {
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
            fn min(self, other: $ty) -> $ty {
                Ord::min(self, other)
            }
            fn max(self, other: $ty) -> $ty {
                Ord::max(self, other)
            }
        }
    };
}

define_summable!(u8);
define_summable!(u16);
define_summable!(u32);
define_summable!(u64);
define_summable!(u128);

define_summable!(i8);
define_summable!(i16);
define_summable!(i32);
define_summable!(i64);
define_summable!(i128);
