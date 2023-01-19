//! ## Problem Statement
//!
//! Find a 3-digit number ABC such that A + B + C = 9, A * B * C = 12, and ...

use solvomatic::{Solvomatic, State};
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Default)]
struct Number([Option<i32>; 5]);

impl State for Number {
    type X = usize;
    type V = i32;

    fn domain() -> Vec<usize> {
        vec![0, 1, 2, 3, 4]
    }
}

impl Index<usize> for Number {
    type Output = Option<i32>;

    fn index(&self, i: usize) -> &Option<i32> {
        &self.0[i]
    }
}

impl IndexMut<usize> for Number {
    fn index_mut(&mut self, i: usize) -> &mut Option<i32> {
        &mut self.0[i]
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for opt_digit in &self.0 {
            if let Some(digit) = opt_digit {
                write!(f, "{}", digit)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut solver = Solvomatic::<Number>::new();

    solver.var(0, 1..9);
    solver.var(1, 1..9);
    solver.var(2, 0..9);
    solver.var(3, 0..9);
    solver.var(4, 0..9);

    solver.simple_constraint("sum", [0, 1], |args| args[0] + args[1] == 7);
    solver.simple_constraint("sum", [0, 2], |args| args[0] + args[1] == 8);
    solver.simple_constraint("sum", [1, 2], |args| args[0] + args[1] == 9);
    solver.simple_constraint("sum", [3, 4], |args| args[0] + args[1] == 1);

    let assignment = solver.solve().unwrap();
    println!("{}", solver.display(&assignment).unwrap());
}
