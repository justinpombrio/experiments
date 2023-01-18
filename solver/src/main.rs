// TODO: temporary
#![allow(unused)]

mod cartesian_prod;

use cartesian_prod::cartesian_prod;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

pub trait Var: Hash + Eq + Display + Clone {}
pub trait Value: PartialEq + Clone {}

impl Var for String {}
impl Value for String {}

#[derive(Debug, Clone)]
struct Domain<X: Var>(HashSet<X>);

/// Constants union cross product of Components
#[derive(Debug, Clone)]
pub struct Assignment<X: Var, V: Value> {
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

pub trait ConstraintTrait<X: Var, V: Value>: Debug {
    fn params(&self) -> &[X];
    fn pred(&self, args: Vec<Option<V>>) -> bool;
}

pub struct Constraint<X: Var, V: Value>(Box<dyn ConstraintTrait<X, V>>);

/*
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
*/

// TODO: debug
pub struct Unsatisfiable<'a, X: Var, V: Value> {
    component: Component<X, V>,
    constraint: &'a dyn ConstraintTrait<X, V>,
}

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
    fn new() -> Assignment<X, V> {
        Assignment {
            domain: Domain::new(),
            constants: Mapping::new(),
            components: Vec::new(),
        }
    }

    fn possibilities(&self) -> usize {
        self.components.iter().map(|a| a.mappings.len()).product()
    }

    fn size(&self) -> usize {
        self.components.iter().map(|a| a.mappings.len()).sum()
    }

    fn assign_var(&mut self, x: X, values: impl IntoIterator<Item = V>) {
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

    fn apply_constraint<'a>(
        &mut self,
        constraint: &'a Constraint<X, V>,
    ) -> Result<(), Unsatisfiable<'a, X, V>> {
        let (disj_comps, shared_comps) = self.components.drain(..).partition::<Vec<_>, _>(|c| {
            !constraint.0.params().iter().any(|x| c.domain.contains(x))
        });
        let mut shared_comp = Component::merge(shared_comps);
        let shared_comp_old = shared_comp.clone(); // for debugging
        shared_comp.retain(|mapping| {
            let args = constraint
                .0
                .params()
                .iter()
                .map(|x| self.constants.get(x).or_else(|| mapping.get(x)))
                .collect::<Vec<_>>();
            constraint.0.pred(args)
        });
        if shared_comp.mappings.len() == 0 {
            return Err(Unsatisfiable {
                component: shared_comp_old,
                constraint: constraint.0.deref() as &dyn ConstraintTrait<X, V>,
            });
        } else {
            for (x, val) in shared_comp.factor_constants().0.into_iter() {
                self.constants.insert(x, val);
            }
        }
        self.components.extend(disj_comps);
        self.components.push(shared_comp);
        Ok(())
    }
}

pub struct Solver<X: Var, V: Value> {
    variables: Vec<(X, Vec<V>)>,
    constraints: Vec<Constraint<X, V>>,
}

impl<X: Var, V: Value> Solver<X, V> {
    pub fn new() -> Solver<X, V> {
        Solver {
            variables: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn add_var(&mut self, x: X, values: impl IntoIterator<Item = V>) {
        self.variables
            .push((x, values.into_iter().collect::<Vec<_>>()));
    }

    pub fn add_constraint(&mut self, constraint: Constraint<X, V>) {
        self.constraints.push(constraint);
    }

    pub fn solve(&mut self) -> Result<Assignment<X, V>, Unsatisfiable<X, V>> {
        let mut assignment = Assignment::new();

        while !self.variables.is_empty() {
            let mut choices = Vec::new();
            for (i, (x, vals)) in self.variables.iter().enumerate() {
                let mut new_assignment = assignment.clone();
                for constraint in &self.constraints {
                    new_assignment.apply_constraint(constraint)?;
                }
                choices.push((i, new_assignment));
            }
            let (best_var_index, best_assignment) =
                choices.into_iter().min_by_key(|(_, a)| a.size()).unwrap();
            self.variables.remove(best_var_index);
            assignment = best_assignment;
        }

        Ok(assignment)
    }
}

fn main() {}
