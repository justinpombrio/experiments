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
use std::default::Default;
use std::time::Instant;

pub mod constraints;

pub use state::State;
pub use table::Table;

/// Solves puzzles in much the same way that hitting them with a brick doesn't.
pub struct Solvomatic<S: State> {
    table: Table<S>,
    constraints: Vec<DynConstraint<S>>,
    config: Config,
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
    apply: Box<dyn Fn(&mut Table<S>) -> Result<bool, ()>>,
    done: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Log when each step completed (default true)
    pub log_steps: bool,
    /// Log when a constraint is completed
    pub log_completed: bool,
    /// Log how long each step took
    pub log_elapsed: bool,
    /// Log intermediate states (can be very large!)
    pub log_states: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            log_steps: true,
            log_completed: false,
            log_elapsed: false,
            log_states: false,
        }
    }
}

impl<S: State> Solvomatic<S> {
    /// Construct an empty solver. Call `var()` and `constraint()` to give it variables and
    /// constraints, then `solve()` to solve for them.
    pub fn new() -> Solvomatic<S> {
        Solvomatic {
            table: Table::new(),
            constraints: Vec::new(),
            config: Config::default(),
        }
    }

    pub fn config(&mut self) -> &mut Config {
        &mut self.config
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
            done: false,
        });
    }

    /// Solves the constraints! Returns `Err(Unsatisfiable)` if it discovers that the constraints
    /// are not, in fact, possible to satisfy. Otherwise, call `.table()` to see the solution(s).
    pub fn solve(&mut self) -> Result<(), Unsatisfiable<S>> {
        let start_time = Instant::now();

        self.table = self.apply_constraints(self.table.clone())?;
        while self.table.num_sections() > 1 && self.table.possibilities() > 1 {
            self.step()?;
        }
        self.table.merge_constants();

        println!("time: {}ms", start_time.elapsed().as_millis());

        Ok(())
    }

    /// Apply one step of solving. It's important to `apply_constraints()` _before_ the first step
    /// though!
    fn step(&mut self) -> Result<(), Unsatisfiable<S>> {
        let start_time = Instant::now();

        let step_num = self.table.num_columns() - self.table.num_sections();
        if self.config.log_steps {
            println!(
                "Step {:2}: size = {:4} possibilities = {}",
                step_num,
                self.table.size(),
                self.table.possibilities(),
            );
        }

        // Mark completed constraints as done
        self.mark_completed_constraints();

        // Merge all constant sections together
        self.table.merge_constants();

        // Consider merging all combinations of two Sections of the table
        if self.table.num_sections() > 1 {
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
        }

        // Log how long it took
        if self.config.log_elapsed {
            let elapsed_time = start_time.elapsed().as_millis();
            println!("  elapsed: {:5?}ms", elapsed_time);
        }

        Ok(())
    }

    /// Repeatedly apply all constraints until that stops having any effect.
    fn apply_constraints(&self, mut table: Table<S>) -> Result<Table<S>, Unsatisfiable<S>> {
        let mut last_size = table.size() + 1;
        while table.size() < last_size {
            last_size = table.size();
            for constraint in &self.constraints {
                if constraint.done {
                    continue;
                }
                match (constraint.apply)(&mut table) {
                    Ok(_) => (),
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

    /// Mark constraints that will _always_ hold as done.
    pub fn mark_completed_constraints(&mut self) {
        for constraint in &mut self.constraints {
            if constraint.done {
                continue;
            }
            if (constraint.apply)(&mut self.table.clone()) == Ok(true) {
                if self.config.log_completed {
                    println!(
                        "  completed constraint {} {:?}",
                        constraint.name, constraint.params
                    );
                }
                constraint.done = true;
            }
        }
    }

    /// The current table of possibilities.
    pub fn table(&self) -> &Table<S> {
        &self.table
    }
}
