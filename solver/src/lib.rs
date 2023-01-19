mod cartesian_prod;

use cartesian_prod::cartesian_prod;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

pub trait Var: fmt::Debug + Hash + Eq + fmt::Display + Clone {}
pub trait Value: fmt::Debug + PartialEq + Clone {}

impl Var for String {}
impl Value for String {}

#[derive(Debug, Clone)]
struct Domain<X: Var>(HashSet<X>);

pub trait DomainTrait<X: Var, V: Value> {
    fn domain(&self) -> &[X];
    fn display(&self, mapping: &Mapping<X, V>) -> String;
}

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
pub struct Mapping<X: Var, V: Value>(HashMap<X, V>);

struct Constraint<X: Var, V: Value> {
    name: String,
    params: Vec<X>,
    pred: Box<dyn Fn(Vec<Option<V>>) -> bool>,
}

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

#[derive(Debug, Clone)]
pub struct Unsatisfiable<X: Var, V: Value> {
    component: Component<X, V>,
    constraint: (String, Vec<X>),
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

    fn contains(&self, x: &X) -> bool {
        self.0.contains(x)
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

    pub fn get(&self, x: &X) -> Option<V> {
        self.0.get(x).cloned()
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

    fn display(&self, domain: &impl DomainTrait<X, V>) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        for line in domain.display(&self).lines() {
            write!(&mut out, "    {}", line).unwrap();
        }
        out
    }
}

impl<X: Var, V: Value> Component<X, V> {
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

    fn is_trivial(&self) -> bool {
        self.mappings.len() == 1 && self.mappings[0].0.is_empty()
    }

    fn display(&self, domain: &impl DomainTrait<X, V>) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        for mapping in &self.mappings {
            writeln!(out, "{}", mapping.display(domain)).unwrap();
        }
        out
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

    fn display(&self, domain: &impl DomainTrait<X, V>) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        writeln!(out, "{}", self.constants.display(domain)).unwrap();
        for component in &self.components {
            writeln!(out, "\nAnd one of:\n").unwrap();
            writeln!(out, "{}", component.display(domain)).unwrap();
        }
        out
    }

    pub fn possibilities(&self) -> usize {
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

        //println!("Assigned var {:#?} to get {:#?}", x, self);
        self.components.push(Component {
            domain: Domain::singleton(x.clone()),
            mappings: values
                .into_iter()
                .map(|val| Mapping::singleton(x.clone(), val))
                .collect::<Vec<_>>(),
        });
    }

    fn apply_constraint<'a>(
        &mut self,
        constraint: &Constraint<X, V>,
    ) -> Result<(), Unsatisfiable<X, V>> {
        let (disj_comps, shared_comps) = self
            .components
            .drain(..)
            .partition::<Vec<_>, _>(|c| !constraint.params.iter().any(|x| c.domain.contains(x)));
        let mut shared_comp = Component::merge(shared_comps);
        let shared_comp_old = shared_comp.clone(); // for debugging
        shared_comp.retain(|mapping| {
            let args = constraint
                .params
                .iter()
                .map(|x| self.constants.get(x).or_else(|| mapping.get(x)))
                .collect::<Vec<_>>();
            (constraint.pred)(args)
        });
        if shared_comp.mappings.len() == 0 {
            return Err(Unsatisfiable {
                component: shared_comp_old,
                constraint: (constraint.name.clone(), constraint.params.clone()),
            });
        } else {
            for (x, val) in shared_comp.factor_constants().0.into_iter() {
                self.constants.insert(x, val);
            }
        }
        self.components.extend(disj_comps);
        if !shared_comp.is_trivial() {
            self.components.push(shared_comp);
        }
        //println!("Added constraint {:#?} to get {:#?}", constraint, self);
        Ok(())
    }
}

pub struct Solvomatic<X: Var, V: Value, D: DomainTrait<X, V>> {
    domain: D,
    variables: Vec<(X, Vec<V>)>,
    constraints: Vec<Constraint<X, V>>,
}

impl<X: Var, V: Value, D: DomainTrait<X, V>> Solvomatic<X, V, D> {
    pub fn new(domain: D) -> Solvomatic<X, V, D> {
        Solvomatic {
            domain,
            variables: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn var(&mut self, x: X, values: impl IntoIterator<Item = V>) {
        self.variables
            .push((x, values.into_iter().collect::<Vec<_>>()));
    }

    pub fn simple_constraint(
        &mut self,
        name: impl Into<String>,
        params: impl IntoIterator<Item = X>,
        predicate: impl Fn(Vec<V>) -> bool + 'static,
    ) {
        self.constraints.push(Constraint {
            name: name.into(),
            params: params.into_iter().collect::<Vec<_>>(),
            pred: Box::new(move |args| {
                let mut unwrapped_args = Vec::with_capacity(args.len());
                for arg in args {
                    if let Some(arg) = arg {
                        unwrapped_args.push(arg);
                    } else {
                        // We're simple, meaning that if any arg is unknown, the constraint might
                        // hold.
                        return true;
                    }
                }
                predicate(unwrapped_args)
            }),
        });
    }

    pub fn constraint(
        &mut self,
        name: impl Into<String>,
        params: impl IntoIterator<Item = X>,
        predicate: impl Fn(Vec<Option<V>>) -> bool + 'static,
    ) {
        self.constraints.push(Constraint {
            name: name.into(),
            params: params.into_iter().collect::<Vec<_>>(),
            pred: Box::new(predicate),
        });
    }

    pub fn solve(&mut self) -> Result<Assignment<X, V>, Unsatisfiable<X, V>> {
        let mut assignment = Assignment::new();

        while !self.variables.is_empty() {
            let mut choices = Vec::new();
            for (i, (x, vals)) in self.variables.iter().enumerate() {
                let mut new_assignment = assignment.clone();
                new_assignment.assign_var(x.clone(), vals.clone());
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

    pub fn display(&self, assignment: &Assignment<X, V>) -> String {
        use std::fmt::Write;

        let mut out = String::new();
        writeln!(&mut out, "Result:").unwrap();
        writeln!(&mut out).unwrap();
        write!(&mut out, "{}", assignment.display(&self.domain)).unwrap();
        out
    }
}

impl Var for char {}
impl Value for i8 {}
