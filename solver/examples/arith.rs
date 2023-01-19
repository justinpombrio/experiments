//! ## Problem Statement
//!
//! Find a 3-digit number ABC such that A + B + C = 9, A * B * C = 12, and ...

use solvomatic::{DomainTrait, Mapping, Solvomatic, Value, Var};

struct SimpleDomain<X: Var, V: Value> {
    domain: Vec<X>,
    display: fn(&Mapping<X, V>) -> String,
}

impl<X: Var, V: Value> DomainTrait<X, V> for SimpleDomain<X, V> {
    fn domain(&self) -> &[X] {
        &self.domain
    }

    fn display(&self, mapping: &Mapping<X, V>) -> String {
        (self.display)(mapping)
    }
}

fn main() {
    fn display_number(mapping: &Mapping<char, i8>) -> String {
        use std::fmt::Write;

        fn write_digit(s: &mut String, digit: Option<i8>) {
            if let Some(digit) = digit {
                write!(s, "{}", digit).unwrap();
            } else {
                write!(s, "_").unwrap();
            }
        }

        let mut s = String::new();
        for letter in "ABCDE".chars() {
            write_digit(&mut s, mapping.get(&letter));
        }
        s
    }
    let five_digit_number = SimpleDomain {
        domain: vec!['A', 'B', 'C', 'D', 'E'],
        display: display_number,
    };

    let mut solver = Solvomatic::new(five_digit_number);

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
    println!("{}", solver.display(&assignment));
}
