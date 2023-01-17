// TODO: temporary
#![allow(unused)]

mod cartesian_prod;

use cartesian_prod::cartesian_prod;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait Var: Hash + Eq + Display + Clone {}
pub trait Value: PartialEq + Clone {}

impl Var for String {}
impl Value for String {}

#[derive(Debug, Clone)]
struct Assignment<X: Var, V: Value> {
    is_empty: bool,
    domain: HashSet<X>,
    constants: HashMap<X, V>,
    components: Vec<Component<X, V>>,
}

// INVARIANT: never empty
#[derive(Debug, Clone)]
struct Component<X: Var, V: Value> {
    domain: HashSet<X>,
    mappings: Vec<Mapping<X, V>>,
}

// INVARIANT: never empty
#[derive(Debug, Clone)]
struct Mapping<X: Var, V: Value>(HashMap<X, V>);

impl<X: Var, V: Value> Assignment<X, V> {
    fn possibilities(&self) -> usize {
        self.components.iter().map(|a| a.mappings.len()).product()
    }

    fn size(&self) -> usize {
        self.components.iter().map(|a| a.mappings.len()).sum()
    }

    pub fn add_var(&mut self, x: X, values: Vec<V>) {
        if self.domain.contains(&x) {
            panic!(
                "Variable '{}' is already in the Assignment, and can't be added again.",
                x
            );
        }

        let mut domain = HashSet::new();
        domain.insert(x.clone());
        let mappings = values
            .into_iter()
            .map(|val| Mapping::new(x.clone(), val))
            .collect::<Vec<_>>();
        self.components.push(Component { domain, mappings });
    }

    pub fn apply_constraint(&mut self, constraint: Constraint<X, V>) {
        let (disj_comps, shared_comps) = self
            .components
            .drain(..)
            .partition::<Vec<_>, _>(|c| !c.domain.is_disjoint(&constraint.domain));
        let shared_comp = merge_components(shared_comps);
        unimplemented!();
        self.components.extend(disj_comps);
        self.components.push(shared_comp);
    }
}

impl<X: Var, V: Value> Component<X, V> {
    fn new(domain: HashSet<X>) -> Component<X, V> {
        Component {
            domain,
            mappings: Vec::new(),
        }
    }
}

impl<X: Var, V: Value> Mapping<X, V> {
    fn new(x: X, val: V) -> Mapping<X, V> {
        let mut map = HashMap::new();
        map.insert(x, val);
        Mapping(map)
    }
}

fn merge_components<X: Var, V: Value>(components: Vec<Component<X, V>>) -> Component<X, V> {
    let mut domain = HashSet::new();
    let mut component_mappings = Vec::new();
    for component in components {
        domain.extend(component.domain);
        component_mappings.push(component.mappings);
    }

    let mut mappings = Vec::new();
    for mappings_iter in cartesian_prod(&component_mappings) {
        let mut mapping: Mapping<X, V> = Mapping(HashMap::new());
        for submapping in mappings_iter {
            for (key, val) in &submapping.0 {
                mapping.0.insert(key.clone(), val.clone());
            }
        }
        mappings.push(mapping);
    }

    Component { domain, mappings }
}

/// A constraint, saying that the values that certain variables have obey some predicate.
pub struct Constraint<X: Var, V: Value> {
    /// The set of variables that are constrained.
    domain: HashSet<X>,
    /// The predicate, saying whether this constraint is satisfied when the variables `self.vars()`
    /// are given the values in the function arg (`Vec<Option<V>>`). `None` represents unspecified
    /// values; `pred` must return true if there is _any_ assignment of them that would yield true.
    pred: Box<dyn Fn(Vec<Option<V>>) -> bool>,
}

fn main() {}
