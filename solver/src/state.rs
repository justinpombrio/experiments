use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

pub trait State: fmt::Debug + 'static {
    type Var: fmt::Debug + Hash + Eq + Ord + Clone + 'static;
    type Value: fmt::Debug + Hash + Eq + Ord + Clone + 'static;

    fn display(f: &mut String, state: &HashMap<Self::Var, Self::Value>) -> fmt::Result {
        use fmt::Write;
        write!(f, "{:#?}", state)
    }
}
