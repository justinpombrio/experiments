mod cartesian_prod;
mod writer;

use cartesian_prod::cartesian_prod;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::ops::{Index, IndexMut};

pub trait Var: fmt::Debug + Hash + Eq + Clone {}
pub trait Value: fmt::Debug + PartialEq + Clone {}

impl<X: fmt::Debug + Hash + Eq + Clone> Var for X {}
impl<V: fmt::Debug + PartialEq + Clone> Value for V {}

pub trait State:
    Clone + Default + fmt::Display + Index<Self::X, Output = Option<Self::V>> + IndexMut<Self::X>
{
    type X: Var + 'static;
    type V: Value + 'static;

    fn domain() -> Vec<Self::X>;
}

#[derive(Debug, Clone)]
struct Domain<X: Var>(HashSet<X>);

/// Constants union cross product of Components
#[derive(Debug, Clone)]
pub struct Assignment<S: State> {
    domain: Domain<S::X>,
    constants: S,
    components: Vec<Component<S>>,
}

/// Union of States
// INVARIANT: never empty
#[derive(Debug, Clone)]
struct Component<S: State> {
    domain: Domain<S::X>,
    states: Vec<S>,
}

struct Constraint<S: State> {
    name: String,
    params: Vec<S::X>,
    pred: Box<dyn Fn(Vec<Option<S::V>>) -> bool>,
}

/*
/// A constraint, saying that the values that certain variables have obey some predicate.
pub struct Constraint<S: State> {
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
pub struct Unsatisfiable<S: State> {
    component: Component<S>,
    constraint: (String, Vec<S::X>),
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

fn merge_states<S: State>(states: impl IntoIterator<Item = S>) -> S {
    let mut result = S::default();
    for state in states {
        for x in S::domain() {
            if let Some(val) = &state[x.clone()] {
                result[x.clone()] = Some(val.clone());
            }
        }
    }
    result
}

impl<S: State> Component<S> {
    fn merge(components: impl IntoIterator<Item = Component<S>>) -> Component<S> {
        let (domain_list, states_list) = components
            .into_iter()
            .map(|comp| (comp.domain, comp.states))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        Component {
            domain: Domain::merge(domain_list),
            states: cartesian_prod(&states_list)
                .map(merge_states)
                .collect::<Vec<_>>(),
        }
    }

    fn retain(&mut self, pred: impl Fn(&S) -> bool) {
        self.states.retain(pred)
    }

    fn factor_constants(&mut self) -> S {
        let mut constants = S::default();
        for x in &self.domain.0 {
            let val = &self.states[0][x.clone()];
            if self.states.iter().all(|s| &s[x.clone()] == val) {
                constants[x.clone()] = val.clone();
                self.states.iter_mut().for_each(|s| s[x.clone()] = None);
            }
        }
        constants
    }

    fn is_trivial(&self) -> bool {
        if self.states.len() != 1 {
            return false;
        }
        for x in S::domain() {
            if self.states[0][x.clone()].is_some() {
                return false;
            }
        }
        true
    }
}

impl<S: State> fmt::Display for Component<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for state in &self.states {
            write!(f, "{}", state)?;
        }
        Ok(())
    }
}

impl<S: State> Assignment<S> {
    fn new() -> Assignment<S> {
        Assignment {
            domain: Domain::new(),
            constants: S::default(),
            components: Vec::new(),
        }
    }

    pub fn possibilities(&self) -> usize {
        self.components.iter().map(|a| a.states.len()).product()
    }

    fn size(&self) -> usize {
        self.components.iter().map(|a| a.states.len()).sum()
    }

    fn assign_var(&mut self, x: S::X, values: impl IntoIterator<Item = S::V>) {
        if self.domain.contains(&x) {
            panic!(
                "Variable '{:?}' is already in the Assignment, and can't be added again.",
                x
            );
        }

        //println!("Assigned var {:#?} to get {:#?}", x, self);
        self.components.push(Component {
            domain: Domain::singleton(x.clone()),
            states: values
                .into_iter()
                .map(|val| {
                    let mut state = S::default();
                    state[x.clone()] = Some(val);
                    state
                })
                .collect::<Vec<_>>(),
        });
    }

    fn apply_constraint<'a>(&mut self, constraint: &Constraint<S>) -> Result<(), Unsatisfiable<S>> {
        let (disj_comps, shared_comps) = self
            .components
            .drain(..)
            .partition::<Vec<_>, _>(|c| !constraint.params.iter().any(|x| c.domain.contains(x)));
        let mut shared_comp = Component::merge(shared_comps);
        let shared_comp_old = shared_comp.clone(); // for debugging
        shared_comp.retain(|state| {
            let args = constraint
                .params
                .iter()
                .map(|x| {
                    self.constants[x.clone()]
                        .as_ref()
                        .or_else(|| state[x.clone()].as_ref())
                        .cloned()
                })
                .collect::<Vec<_>>();
            (constraint.pred)(args)
        });
        if shared_comp.states.len() == 0 {
            return Err(Unsatisfiable {
                component: shared_comp_old,
                constraint: (constraint.name.clone(), constraint.params.clone()),
            });
        } else {
            let constants = shared_comp.factor_constants();
            for x in S::domain() {
                if let Some(val) = &constants[x.clone()] {
                    self.constants[x.clone()] = Some(val.clone());
                }
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

impl<S: State> fmt::Display for Assignment<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.constants)?;
        for component in &self.components {
            writeln!(f, "\nAnd one of:\n")?;
            write!(f, "{}", component)?;
        }
        Ok(())
    }
}

pub struct Solvomatic<S: State> {
    variables: Vec<(S::X, Vec<S::V>)>,
    constraints: Vec<Constraint<S>>,
}

impl<S: State> Solvomatic<S> {
    pub fn new() -> Solvomatic<S> {
        Solvomatic {
            variables: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn var(&mut self, x: S::X, values: impl IntoIterator<Item = S::V>) {
        self.variables
            .push((x, values.into_iter().collect::<Vec<_>>()));
    }

    pub fn simple_constraint(
        &mut self,
        name: impl Into<String>,
        params: impl IntoIterator<Item = S::X>,
        predicate: impl Fn(Vec<S::V>) -> bool + 'static,
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
        params: impl IntoIterator<Item = S::X>,
        predicate: impl Fn(Vec<Option<S::V>>) -> bool + 'static,
    ) {
        self.constraints.push(Constraint {
            name: name.into(),
            params: params.into_iter().collect::<Vec<_>>(),
            pred: Box::new(predicate),
        });
    }

    pub fn solve(&mut self) -> Result<Assignment<S>, Unsatisfiable<S>> {
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

    pub fn display(&self, assignment: &Assignment<S>) -> Result<String, fmt::Error> {
        use std::fmt::Write;

        let mut out = String::new();
        writeln!(&mut out, "Result:")?;
        writeln!(&mut out)?;
        write!(&mut out, "{}", assignment)?;
        Ok(out)
    }
}
