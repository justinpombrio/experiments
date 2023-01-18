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
struct Domain<X: Var>(HashSet<X>);

/// Cross product of Components
#[derive(Debug, Clone)]
struct Assignment<X: Var, V: Value> {
    is_empty: bool,
    domain: Domain<X>,
    constants: Mapping<X, V>,
    components: Vec<Component<X, V>>,
}

/// Union of Mappings
// INVARIANT: never empty
#[derive(Debug, Clone)]
struct Component<X: Var, V: Value> {
    domain: Domain<X>,
    mappings: Vec<Mapping<X, V>>,
}

/// Map from var (X) to value (V)
// INVARIANT: never empty
#[derive(Debug, Clone)]
struct Mapping<X: Var, V: Value>(HashMap<X, V>);

impl<X: Var> Domain<X> {
    fn new() -> Domain<X> {
        Domain(HashSet::new())
    }

    fn singleton(x: X) -> Domain<X> {
        let mut set = HashSet::new();
        set.insert(x);
        Domain(set)
    }

    fn insert(&mut self, x: X) {
        self.0.insert(x);
    }

    fn contains(&self, x: &X) -> bool {
        self.0.contains(x)
    }

    fn is_disjoint(&self, other: &Domain<X>) -> bool {
        self.0.is_disjoint(&other.0)
    }

    fn merge(domains: impl IntoIterator<Item = Domain<X>>) -> Domain<X> {
        let mut set = HashSet::new();
        for domain in domains {
            set.extend(domain.0);
        }
        Domain(set)
    }
}

impl<X: Var, V: Value> Mapping<X, V> {
    fn new() -> Mapping<X, V> {
        Mapping(HashMap::new())
    }

    fn singleton(x: X, val: V) -> Mapping<X, V> {
        let mut map = HashMap::new();
        map.insert(x, val);
        Mapping(map)
    }

    fn insert(&mut self, x: X, val: V) {
        self.0.insert(x, val);
    }

    fn remove(&mut self, x: &X) {
        self.0.remove(x);
    }

    fn merge(mappings: impl IntoIterator<Item = Mapping<X, V>>) -> Mapping<X, V> {
        let mut map = HashMap::new();
        for mapping in mappings {
            map.extend(mapping.0.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        Mapping(map)
    }

    fn get(&self, x: &X) -> Option<V> {
        self.0.get(x).cloned()
    }
}

impl<X: Var, V: Value> Component<X, V> {
    fn new(domain: Domain<X>) -> Component<X, V> {
        Component {
            domain,
            mappings: Vec::new(),
        }
    }

    fn merge(components: impl IntoIterator<Item = Component<X, V>>) -> Component<X, V> {
        let (domain_list, mappings_list) = components
            .into_iter()
            .map(|comp| (comp.domain, comp.mappings))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        Component {
            domain: Domain::merge(domain_list),
            mappings: cartesian_prod(&mappings_list)
                .map(|ms| Mapping::merge(ms))
                .collect::<Vec<_>>(),
        }
    }

    fn retain(&mut self, pred: impl Fn(&Mapping<X, V>) -> bool) {
        self.mappings.retain(pred)
    }

    fn factor_constants(&mut self) -> Mapping<X, V> {
        let mut constants = Mapping::new();
        for x in &self.domain.0 {
            let val = &self.mappings[0].0[x];
            if self.mappings.iter().all(|m| &m.0[x] == val) {
                constants.insert(x.clone(), val.clone());
                self.mappings.iter_mut().for_each(|m| m.remove(x));
            }
        }
        constants
    }
}

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

        self.components.push(Component {
            domain: Domain::singleton(x.clone()),
            mappings: values
                .into_iter()
                .map(|val| Mapping::singleton(x.clone(), val))
                .collect::<Vec<_>>(),
        })
    }

    pub fn apply_constraint(&mut self, constraint: Constraint<X, V>) {
        let (disj_comps, shared_comps) = self
            .components
            .drain(..)
            .partition::<Vec<_>, _>(|c| !c.domain.is_disjoint(&constraint.domain));
        let mut shared_comp = Component::merge(shared_comps);
        shared_comp.retain(|mapping| {
            let args = constraint
                .params
                .iter()
                .map(|x| self.constants.get(x).or_else(|| mapping.get(x)))
                .collect::<Vec<_>>();
            (constraint.pred)(args)
        });
        if shared_comp.mappings.len() == 0 {
            self.is_empty = true;
        } else {
            for (x, val) in shared_comp.factor_constants().0.into_iter() {
                self.constants.insert(x, val);
            }
        }
        self.components.extend(disj_comps);
        self.components.push(shared_comp);
    }
}

/// A constraint, saying that the values that certain variables have obey some predicate.
pub struct Constraint<X: Var, V: Value> {
    /// The set of variables that are constrained.
    domain: Domain<X>,
    /// The set of variables that are constrained, in the same order as they will be passed to
    /// `pred`.
    params: Vec<X>,
    /// The predicate, saying whether this constraint is satisfied when the variables `self.vars()`
    /// are given the values in the function arg (`Vec<Option<V>>`). `None` represents unspecified
    /// values; `pred` must return true if there is _any_ assignment of them that would yield true.
    pred: Box<dyn Fn(Vec<Option<V>>) -> bool>,
}

impl<X: Var, V: Value> Constraint<X, V> {
    fn satisfied_by(&self, mapping: &Mapping<X, V>) -> bool {
        let args = self
            .params
            .iter()
            .map(|x| mapping.get(x))
            .collect::<Vec<_>>();
        (self.pred)(args)
    }
}

fn main() {}
