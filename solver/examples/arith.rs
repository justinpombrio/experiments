//! ## Problem Statement
//!
//! Find a 3-digit number ABC such that A + B + C = 9, A * B * C = 12, and ...

use solvomatic::{Mapping, Solvomatic};
use std::fmt;

fn main() {
    let mut solver = Solvomatic::new(
        |f: &mut String, mapping: &Mapping<char, i8>| -> fmt::Result {
            use std::fmt::Write;

            for letter in "ABCDE".chars() {
                if let Some(digit) = mapping.get(&letter) {
                    write!(f, "{}", digit)?;
                } else {
                    write!(f, "_")?;
                }
            }
            Ok(())
        },
    );

    solver.var('A', 1..9);
    solver.var('B', 1..9);
    solver.var('C', 0..9);
    solver.var('D', 0..9);
    solver.var('E', 0..9);

    solver.simple_constraint("sum", ['A', 'B'], |args| args[0] + args[1] == 7);
    solver.simple_constraint("sum", ['A', 'C'], |args| args[0] + args[1] == 8);
    solver.simple_constraint("sum", ['B', 'C'], |args| args[0] + args[1] == 9);
    solver.simple_constraint("sum", ['D', 'E'], |args| args[0] + args[1] == 1);

    let assignment = solver.solve().unwrap();
    println!("{}", solver.display(&assignment).unwrap());
}
