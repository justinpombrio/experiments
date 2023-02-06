//! Intelligently computed restrictions on the `Value`s that `Var`s can have.

mod ordbag;
mod pred;
mod prod;
mod seq;
mod sum;

use std::fmt::Debug;

pub use ordbag::Bag;
pub use pred::Pred;
pub use prod::Prod;
pub use seq::Seq;
pub use sum::Sum;

/// A constraint on a set of elements `T`. A naive implementation would use a predicate: it would
/// take a `Vec<T>` and return a `bool`. However, there are often far too many possibilities under
/// consideration for that to be feasible to compute. Instead, individual elements `T` are wrapped
/// in `singleton`, unions are combined with `or`, cross products are combined with `and`, and at
/// the end the result is checked with `check`. For example, this set of 60 possibilities:
///
/// ```text
///     A C | B | D
///     ----+---+---
///     1 1 | 2 | 10
///     1 2 | 3 | 20
///     2 1 | 4 | 30
///         | 5 | 40
///         | 6 |
/// ```
///
/// would be combined together into:
///
/// ```text
///       (s(1) * s(1) + s(1) * s(2) + s(2) * s(1))
///     * (s(2) + s(3) + s(4) + s(5) + s(6))
///     * (s(10) + s(20) + s(30) + s(40))
/// ```
///
/// where `s` is `singleton`, `+` is `or`, and `*` is `and`.
pub trait Constraint<T>: 'static {
    /// A "set" of elements. This typically won't actually contain all of the elements! Instead, it
    /// will be a _conservative_ representation. For example, the `Set` for the `Sum` constraint is
    /// a minimum and maximum of the numbers "in the set".
    type Set: Debug + Clone;

    /// A name for this kind of constraint, for debugging purposes.
    const NAME: &'static str;

    /// Construct a set containing just one element.
    fn singleton(&self, index: usize, elem: T) -> Self::Set;
    /// The cross product of two sets.
    fn and(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;
    /// The union of two sets.
    fn or(&self, set_1: Self::Set, set_2: Self::Set) -> Self::Set;

    /// Might the given set satisfy the constraint? If uncertain, return true.
    fn check(&self, set: Self::Set) -> YesNoMaybe;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YesNoMaybe {
    Yes,
    No,
    Maybe,
}

impl YesNoMaybe {
    pub fn and(&self, other: YesNoMaybe) -> YesNoMaybe {
        use YesNoMaybe::{Maybe, No, Yes};

        match (self, other) {
            (Yes, Yes) => Yes,
            (Maybe, Maybe) | (Yes, Maybe) | (Maybe, Yes) => Maybe,
            (No, _) | (_, No) => No,
        }
    }
}
