// TODO: check for emptiness

mod constraints;
mod state;
mod table;

use constraints::Constraint;
use std::collections::HashMap;
use std::fmt;
use table::Table;

pub use constraints::Sum;
pub use state::State;

struct DynConstraint<S: State> {
    name: String,
    params: Vec<S::Var>,
    apply: Box<dyn Fn(&mut Table<S>)>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Unsatisfiable<S: State> {
    state: HashMap<S::Var, S::Value>,
    constraint: (String, Vec<S::Var>),
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

    pub fn constraint<C: Constraint<Element = S::Value>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) {
        let name = C::NAME.to_owned();
        let params = params.into_iter().collect::<Vec<_>>();
        let params_copy = params.clone();
        let apply = Box::new(move |table: &mut Table<S>| {
            table.apply_constraint(&params_copy, &constraint);
        });
        self.constraints.push(DynConstraint {
            name,
            params,
            apply,
        });
    }

    pub fn solve(&mut self) {
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
                            (constraint.apply)(&mut new_table);
                        }
                    }
                    options.push(new_table);
                }
            }

            table = options.into_iter().min_by_key(|t| t.size()).unwrap();
        }

        self.table = table;
    }
}

impl<S: State> fmt::Display for Solvomatic<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        S::display(&mut s, &self.table.state())?;
        write!(f, "{}", s)
    }
}
