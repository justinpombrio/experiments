//! ## Problem Statement
//!
//! Find a 3-digit number ABC such that A + B + C = 9, A * B * C = 12, and ...

use solvomatic::{ConstraintTrait, Solvomatic, Value, Var};
use std::fmt;

struct SimpleConstraint<X: Var, V: Value, D: fmt::Debug> {
    name: &'static str,
    data: D,
    params: Vec<X>,
    pred: fn(args: Vec<Option<V>>, data: &D) -> bool,
}

impl<X: Var, V: Value, D: fmt::Debug> fmt::Debug for SimpleConstraint<X, V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Constraint {}({:?}) on {:?}",
            self.name, self.data, self.params
        )
    }
}

impl<X: Var, V: Value, D: fmt::Debug> ConstraintTrait<X, V> for SimpleConstraint<X, V, D> {
    fn params(&self) -> &[X] {
        &self.params
    }

    fn pred(&self, args: Vec<Option<V>>) -> bool {
        (self.pred)(args, &self.data)
    }
}

fn main() {
    let mut solver = Solvomatic::new();

    solver.add_var('A', 1..9);
    solver.add_var('B', 1..9);
    solver.add_var('C', 0..9);

    fn has_sum(args: Vec<Option<i32>>, expected_sum: &i32) -> bool {
        let mut sum = 0;
        for arg in args {
            if let Some(n) = arg {
                sum += n;
            } else {
                return true;
            }
        }
        sum == *expected_sum
    }
    solver.add_constraint(SimpleConstraint {
        name: "sum",
        data: 7,
        params: vec!['A', 'B'],
        pred: has_sum,
    });
    solver.add_constraint(SimpleConstraint {
        name: "sum",
        data: 8,
        params: vec!['A', 'C'],
        pred: has_sum,
    });
    solver.add_constraint(SimpleConstraint {
        name: "sum",
        data: 9,
        params: vec!['B', 'C'],
        pred: has_sum,
    });

    let assignment = solver.solve().unwrap();
    println!("{:#?}", assignment);
}

/*
struct FunctionConstraint {
    args: Vec<char>,
    pred: fn(Vec<Option<u8>>) -> bool,
}

impl

struct Sum(Vec<char>,
pub trait ConstraintTrait<X: Var, V: Value>: Debug {
    fn params(&self) -> &[X];
    fn pred(&self, args: Vec<Option<V>>) -> bool;
}

fn main() {
    let solver = Solvomatic::new();
    solver.add_var('A', 1..9);
    solver.add_var('B', 1..9);
    solver.add_var('C', 0..9);
    solver.add_constraint(
}
*/
