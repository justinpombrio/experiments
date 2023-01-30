use solvomatic::constraints::{Prod, Sum};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
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

    solver.mapped_constraint(['A', 'B'], |i, n| [2, 1][i] * n, Sum::new(10));
    solver.constraint(['A', 'C'], Sum::new(8));
    solver.constraint(['B', 'C'], Sum::new(9));
    solver.constraint(['D', 'E'], Sum::new(2));
    solver.constraint(['D', 'E'], Prod::new(1));

    solver.solve().unwrap();
    println!("{}", solver);
}
