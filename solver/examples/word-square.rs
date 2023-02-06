//! Find all 4x4 word squares whose diagonals are vowels

use solvomatic::constraints::{Pred, Seq};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct WordSquare;

impl State for WordSquare {
    type Var = (i8, i8);
    type Value = char;

    fn display(f: &mut String, state: &HashMap<(i8, i8), char>) -> fmt::Result {
        use std::fmt::Write;

        fn show_cell(f: &mut String, i: i8, j: i8, state: &HashMap<(i8, i8), char>) -> fmt::Result {
            if let Some(n) = state.get(&(i, j)) {
                write!(f, "{}", n)
            } else {
                write!(f, "_")
            }
        }

        for i in 0..4 {
            for j in 0..4 {
                show_cell(f, i, j, state)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    println!("Finding all 4x4 word squares whose diagonals are vowels.");
    println!();

    let mut solver = Solvomatic::<WordSquare>::new();
    solver.config().log_completed = true;

    let mut all_cells = Vec::new();
    for i in 0..4 {
        for j in 0..4 {
            all_cells.push((i, j));
        }
    }

    // Every cell is a letter, and a letter along a diagonal must be a vowel.
    let diagonals = [
        (0, 0),
        (1, 1),
        (2, 2),
        (3, 3),
        (0, 3),
        (1, 2),
        (2, 1),
        (3, 0),
    ];
    for cell in &all_cells {
        if diagonals.contains(cell) {
            solver.var(*cell, ['a', 'e', 'i', 'o', 'u']);
        } else {
            solver.var(*cell, 'a'..='z');
        }
    }

    // Every row and col forms a word
    let word_of_len_4 = Seq::word_list_file("/usr/share/dict/words", 4).unwrap();
    for i in 0..4 {
        solver.constraint([(i, 0), (i, 1), (i, 2), (i, 3)], word_of_len_4.clone());
    }
    for j in 0..4 {
        solver.constraint([(0, j), (1, j), (2, j), (3, j)], word_of_len_4.clone());
    }

    // WLOG, reflect the word square so that the upper-right cell is less than the lower-left
    // cell.
    solver.constraint([(0, 3), (3, 0)], Pred::new(|[x, y]| x < y));

    solver.solve().unwrap();
    println!("{}", solver.table());
}
