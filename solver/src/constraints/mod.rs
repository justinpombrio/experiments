mod sum;

use crate::state::State;

pub use sum::Sum;

pub trait Constraint<S: State>: 'static {
    type Set: Clone;

    const NAME: &'static str;

    fn new_set(&self, index: usize, elem: S::Value) -> Self::Set;
    fn none(&self) -> Self::Set;
    fn and(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;
    fn andnot(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;
    fn or(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;

    fn params(&self) -> &[S::Var];

    fn check(&self, set: Self::Set) -> bool;
}
