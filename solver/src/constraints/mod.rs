mod prod;
mod sum;

pub use prod::{Mullable, Prod};
pub use sum::{Sum, Summable};

pub trait Constraint<T>: 'static {
    type Set: Clone;

    const NAME: &'static str;

    fn singleton(&self, elem: T) -> Self::Set;
    fn and(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;
    fn or(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;

    fn check(&self, set: Self::Set) -> bool;
}
