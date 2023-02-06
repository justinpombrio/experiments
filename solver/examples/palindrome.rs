//! Find all six letter palindromes in /usr/share/dict/words

use solvomatic::constraints::{Pred, Seq};
use solvomatic::{Solvomatic, State};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
struct Palindrome;

impl State for Palindrome {
    type Var = usize;
    type Value = char;

    fn display(f: &mut String, state: &HashMap<usize, char>) -> fmt::Result {
        use std::fmt::Write;

        fn show_cell(f: &mut String, i: usize, state: &HashMap<usize, char>) -> fmt::Result {
            if let Some(ch) = state.get(&i) {
                write!(f, "{}", ch)
            } else {
                write!(f, "_")
            }
        }

        for i in 0..6 {
            show_cell(f, i, state)?;
        }
        Ok(())
    }
}

fn main() {
    println!("Finding all six letter palindromes in /usr/share/dict/words");
    println!();

    let mut solver = Solvomatic::<Palindrome>::new();
    solver.config().log_completed = true;

    // Every cell is a letter
    solver.var(0, 'a'..='z');
    solver.var(1, 'a'..='z');
    solver.var(2, 'a'..='z');
    solver.var(3, 'a'..='z');
    solver.var(4, 'a'..='z');
    solver.var(5, 'a'..='z');

    // The whole thing is a word
    let word_of_len_6 = Seq::word_list_file("/usr/share/dict/words", 6).unwrap();
    solver.constraint([0, 1, 2, 3, 4, 5], word_of_len_6);

    // It's a palindrome
    solver.constraint([0, 5], Pred::new(|[a, b]| *a == *b));
    solver.constraint([1, 4], Pred::new(|[a, b]| *a == *b));
    solver.constraint([2, 3], Pred::new(|[a, b]| *a == *b));

    solver.solve().unwrap();
    println!("{}", solver.table());
}
