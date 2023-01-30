mod state;
mod table;

use constraints::Constraint;
use std::collections::HashMap;
use std::fmt;
use table::Table;

pub mod constraints;

pub use state::State;

struct DynConstraint<S: State> {
    name: String,
    params: Vec<S::Var>,
    apply: Box<dyn Fn(&mut Table<S>) -> Result<(), ()>>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Unsatisfiable<S: State> {
    state: HashMap<S::Var, S::Value>,
    header: Vec<S::Var>,
    constraint: String,
}

pub struct Solvomatic<S: State> {
    table: Table<S>,
    constraints: Vec<DynConstraint<S>>,
}

impl<S: State> Solvomatic<S> {
    pub fn new() -> Solvomatic<S> {
        Solvomatic {
            table: Table::new(),
            constraints: Vec::new(),
        }
    }

    pub fn var(&mut self, x: S::Var, values: impl IntoIterator<Item = S::Value>) {
        self.table.add_column(x, values);
    }

    pub fn constraint<C: Constraint<S::Value>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var> + 'static,
        constraint: C,
    ) {
        self.mapped_constraint(params, |_, v| v, constraint)
    }

    pub fn mapped_constraint<N, C: Constraint<N>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var> + 'static,
        map: impl Fn(usize, S::Value) -> N + 'static,
        constraint: C,
    ) {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();
        let params_copy = params.clone();
        let apply = Box::new(move |table: &mut Table<S>| {
            table.apply_constraint(&params_copy, &map, &constraint)
        });
        self.constraints.push(DynConstraint {
            name,
            params,
            apply,
        });
    }

    pub fn solve(&mut self) -> Result<(), Unsatisfiable<S>> {
        let mut table = self.table.clone();

        while table.num_sections() > 1 {
            let mut options = Vec::new();
            for i in 0..table.num_sections() - 1 {
                for j in i + 1..table.num_sections() {
                    let mut new_table = table.clone();
                    new_table.merge(i, j);
                    let mut last_size = new_table.size() + 1;
                    while new_table.size() < last_size {
                        last_size = new_table.size();
                        for constraint in &self.constraints {
                            match (constraint.apply)(&mut new_table) {
                                Ok(()) => (),
                                Err(()) => {
                                    return Err(Unsatisfiable {
                                        state: table.state(),
                                        constraint: constraint.name.clone(),
                                        header: constraint.params.clone(),
                                    })
                                }
                            }
                        }
                    }
                    options.push(new_table);
                }
            }

            table = options.into_iter().min_by_key(|t| t.size()).unwrap();
        }

        self.table = table;
        Ok(())
    }

    pub fn size(&self) -> u64 {
        self.table.size()
    }

    pub fn possibilities(&self) -> u64 {
        self.table.possibilities()
    }
}

impl<S: State> fmt::Display for Solvomatic<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        S::display(&mut s, &self.table.state())?;
        write!(f, "{}", s)
    }
}
