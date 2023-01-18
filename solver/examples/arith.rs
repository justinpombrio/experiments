//! ## Problem Statement
//!
//! Find a 3-digit number ABC such that A + B + C = 9, A * B * C = 12, and ...

use solvomatic::{ConstraintTrait, DomainTrait, Mapping, Solvomatic, Value, Var};
use std::fmt;

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

    solver.add_var('A', 1..9);
    solver.add_var('B', 1..9);
    solver.add_var('C', 0..9);
    solver.add_var('D', 0..9);
    solver.add_var('E', 0..9);

    fn has_sum(args: Vec<Option<i8>>, expected_sum: &i8) -> bool {
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
    solver.add_constraint(SimpleConstraint {
        name: "sum",
        data: 1,
        params: vec!['D', 'E'],
        pred: has_sum,
    });

    let assignment = solver.solve().unwrap();
    println!("{}", solver.display(&assignment));
}
