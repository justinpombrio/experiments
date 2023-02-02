//! Some puzzles ask from you a spark of insight, or a delightful recognition.
//!
//! For all the others, there's solvOmatic.
//!
//! TODO: Overview and examples

mod state;
mod table;

// TODO:
// - printing: show column grouping?
// - printing: log vs. stdout? Stdout vs. stderr?
// - more constraints!
// - testing!
// - command line args, including `--log` that prints after each step

use constraints::Constraint;

use std::time::Instant;

pub mod constraints;

pub use state::State;
pub use table::Table;

/// Solves puzzles in much the same way that hitting them with a brick doesn't.
pub struct Solvomatic<S: State> {
    table: Table<S>,
    constraints: Vec<DynConstraint<S>>,
    start_time: Instant,
    last_step_time: Instant,
}

/// The problem was over constrained! Contained is a snapshot of the Table just before a constraint
/// was applied that shrunk that Table's number of possibilities to zero, together with information
/// about that constraint.
#[derive(Debug, Clone)]
pub struct Unsatisfiable<S: State> {
    pub table: Table<S>,
    pub header: Vec<S::Var>,
    pub constraint: String,
}

struct DynConstraint<S: State> {
    name: String,
    params: Vec<S::Var>,
    apply: Box<dyn Fn(&mut Table<S>) -> Result<(), ()>>,
}

impl<S: State> Solvomatic<S> {
    /// Construct an empty solver. Call `var()` and `constraint()` to give it variables and
    /// constraints, then `solve()` to solve for them.
    pub fn new() -> Solvomatic<S> {
        Solvomatic {
            table: Table::new(),
            constraints: Vec::new(),
            start_time: Instant::now(),
            last_step_time: Instant::now(),
        }
    }

    /// Add a new variable, with a set of possible values.
    pub fn var(&mut self, x: S::Var, values: impl IntoIterator<Item = S::Value>) {
        self.table.add_column(x, values);
    }

    /// Add the requirement that the variables `params` must obey `constraint`.
    pub fn constraint<C: Constraint<S::Value>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
        constraint: C,
    ) {
        self.mapped_constraint(params, |_, v| v, constraint)
    }

    /// Add the requirement that the variables `params`, after being `map`ed, must obey
    /// `constraint`.
    pub fn mapped_constraint<N, C: Constraint<N>>(
        &mut self,
        params: impl IntoIterator<Item = S::Var>,
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

    /// Solves the constraints! Returns `Err(Unsatisfiable)` if it discovers that the constraints
    /// are not, in fact, possible to satisfy. Otherwise, call `.table()` to see the solution(s).
    pub fn solve(&mut self) -> Result<(), Unsatisfiable<S>> {
        self.table = self.apply_constraints(self.table.clone())?;

        while self.table.num_sections() > 1 {
            self.step()?;
        }

        Ok(())
    }

    /// Apply one step of solving. It's important to `apply_constraints()` _before_ the first step
    /// though!
    fn step(&mut self) -> Result<(), Unsatisfiable<S>> {
        let step_num = self.table.num_columns() - self.table.num_sections();
        println!(
            "Step {:2}: size = {:4} possibilities = {}",
            step_num,
            self.table.size(),
            self.table.possibilities(),
        );
        let total_time = self.start_time.elapsed().as_millis();
        let elapsed_time = self.last_step_time.elapsed().as_millis();
        self.last_step_time = Instant::now();
        println!(
            "  elapsed: {:5?}ms total: {:5?}ms",
            elapsed_time, total_time
        );
        println!();

        // Consider merging all combinations of two Sections of the table
        let mut options = Vec::new();
        for i in 0..self.table.num_sections() - 1 {
            for j in i + 1..self.table.num_sections() {
                let mut new_table = self.table.clone();
                new_table.merge(i, j);
                new_table = self.apply_constraints(new_table)?;
                options.push(new_table);
            }
        }

        // Merge the two sections that minimize the resulting table size
        self.table = options.into_iter().min_by_key(|t| t.size()).unwrap();
        Ok(())
    }

    /// Repeatedly apply all constraints until that stops having any effect.
    fn apply_constraints(&self, mut table: Table<S>) -> Result<Table<S>, Unsatisfiable<S>> {
        let mut last_size = table.size() + 1;
        while table.size() < last_size {
            last_size = table.size();
            for constraint in &self.constraints {
                match (constraint.apply)(&mut table) {
                    Ok(()) => (),
                    Err(()) => {
                        return Err(Unsatisfiable {
                            table,
                            constraint: constraint.name.clone(),
                            header: constraint.params.clone(),
                        })
                    }
                }
            }
        }
        Ok(table)
    }

    /// The current table of possibilities.
    pub fn table(&self) -> &Table<S> {
        &self.table
    }
}
