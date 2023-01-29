use super::Constraint;

pub struct Sum<N: Clone + 'static> {
    expected_total: N,
    map: Box<dyn Fn(usize, N) -> (N, N)>,
}

impl<N: Clone + 'static> Sum<N> {
    pub fn new(expected_total: N) -> Sum<N> {
        Sum {
            expected_total,
            map: Box::new(|_, n| (n.clone(), n)),
        }
    }

    pub fn new_generic(expected_total: N, map: impl Fn(usize, N) -> (N, N) + 'static) -> Sum<N> {
        Sum {
            expected_total,
            map: Box::new(map),
        }
    }
}

macro_rules! define_sum {
    ($ty:ident) => {
        impl Constraint for Sum<$ty> {
            type Element = $ty;
            type Set = ($ty, $ty);

            const NAME: &'static str = "Sum";

            fn new_set(&self, index: usize, elem: $ty) -> ($ty, $ty) {
                (self.map)(index, elem)
            }

            fn none(&self) -> ($ty, $ty) {
                (0, 0)
            }

            fn and(&self, a: ($ty, $ty), b: ($ty, $ty)) -> ($ty, $ty) {
                (a.0 + b.0, a.1 + b.1)
            }

            fn andnot(&self, a: ($ty, $ty), b: ($ty, $ty)) -> ($ty, $ty) {
                (a.0 - b.0, a.1 - b.1)
            }

            fn or(&self, a: ($ty, $ty), b: ($ty, $ty)) -> ($ty, $ty) {
                (a.0.min(b.0), a.1.max(b.1))
            }

            fn check(&self, set: ($ty, $ty)) -> bool {
                set.0 <= self.expected_total && self.expected_total <= set.1
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
