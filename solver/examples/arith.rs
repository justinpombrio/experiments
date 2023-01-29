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

    solver.constraint(['A', 'B'], Sum::new(7u8));
    solver.constraint(['A', 'C'], Sum::new(8u8));
    solver.constraint(['B', 'C'], Sum::new(9u8));
    solver.constraint(['D', 'E'], Sum::new(1u8));

    solver.solve();
    println!("{}", solver);
}
