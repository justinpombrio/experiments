use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

/// A state is a mapping from `State::Var` to `State::Value`.
///
/// This trait does not actually contain a state, it just declares the types for `Var` and `Value`.
/// As such, you typically want to implement it as an empty struct: `struct MyState; ... impl State
/// for MyState { ... }`.
pub trait State: fmt::Debug + 'static {
    type Var: fmt::Debug + Hash + Eq + Ord + Clone + 'static;
    type Value: fmt::Debug + Hash + Eq + Ord + Clone + 'static;

    /// Print the state nicely for debugging. None all `Var`s will be present; those that aren't
    /// should be printed as a "blank" (of some sort).
    fn display(f: &mut String, state: &HashMap<Self::Var, Self::Value>) -> fmt::Result {
        use fmt::Write;
        write!(f, "{:#?}", state)
    }
}
