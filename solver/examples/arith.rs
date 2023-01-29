use solvomatic::{Solvomatic, State, Sum};
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

    solver.constraint(Sum::new_generic(['A', 'B'], 10, |i, n| [2, 1][i] * n));
    solver.constraint(Sum::new(['A', 'C'], 8));
    solver.constraint(Sum::new(['B', 'C'], 9));
    solver.constraint(Sum::new(['D', 'E'], 1));

    solver.solve();
    println!("{}", solver);
}
