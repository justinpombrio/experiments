use solvomatic::{RangeRing, Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

struct PuzzleState;

impl State for PuzzleState {
    type Var = char;
    type Value = u8;

    fn display(f: &mut String, state: &HashMap<char, u8>) -> fmt::Result {
        use std::fmt::Write;

        for letter in "ABCDE".chars() {
            if let Some(digit) = state.get(&letter) {
                write!(f, "{}", digit)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut solver = Solvomatic::<PuzzleState>::new();

    solver.var('A', 1..9);
    solver.var('B', 0..9);
    solver.var('C', 0..9);
    solver.var('D', 0..9);
    solver.var('E', 0..9);

    solver.generic_constraint(
        "sumAB",
        ['A', 'B'],
        |_, n| RangeRing::new(n),
        |r| r.0 <= 7 && r.1 >= 7,
    );
    solver.generic_constraint(
        "sumAB",
        ['A', 'C'],
        |_, n| RangeRing::new(n),
        |r| r.0 <= 8 && r.1 >= 8,
    );
    solver.generic_constraint(
        "sumAB",
        ['B', 'C'],
        |_, n| RangeRing::new(n),
        |r| r.0 <= 9 && r.1 >= 9,
    );
    solver.generic_constraint(
        "sumAB",
        ['D', 'E'],
        |_, n| RangeRing::new(n),
        |r| r.0 <= 1 && r.1 >= 1,
    );

    // solver.simple_constraint("sumAB", ['A', 'B'], |args| args[0] + args[1] == 7);
    // solver.simple_constraint("sumAC", ['A', 'C'], |args| args[0] + args[1] == 8);
    // solver.simple_constraint("sumBC", ['B', 'C'], |args| args[0] + args[1] == 9);
    // solver.simple_constraint("sumDE", ['D', 'E'], |args| args[0] + args[1] == 1);

    solver.solve();
    println!("{}", solver);
}
