mod partition;
mod word;

use partition::Partition;
use std::fs::File;
use std::io::{BufRead, BufReader};
use word::{color, Coloring, Word};

struct GameState {
    guesses: Vec<(Word, Coloring)>,
    remaining_words: Vec<Word>,
    allowed_guesses: Vec<Word>,
}

fn read_words(filename: &str) -> Vec<Word> {
    let reader = BufReader::new(File::open(filename).unwrap());
    let mut words = vec![];
    for line in reader.lines() {
        words.push(Word::from_str(&line.unwrap().to_uppercase()));
    }
    words
}

impl GameState {
    fn new() -> GameState {
        let possible_solutions = read_words("possible_solutions.txt");
        let allowed_guesses = read_words("allowed_guesses.txt");
        GameState {
            guesses: vec![],
            remaining_words: possible_solutions,
            allowed_guesses,
        }
    }

    fn insert_feedback(&mut self, guess: Word, coloring: Coloring) {
        self.guesses.push((guess, coloring));

        let mut partition = Partition::new();
        for solution in &self.remaining_words {
            let coloring = color(guess, *solution);
            partition.insert(*solution, coloring);
        }
        self.remaining_words = match partition.extract_subset(coloring) {
            None => {
                println!("Liar liar!");
                println!("That's not a possible coloring for the game history so far.");
                println!("Have a bad day.");
                std::process::exit(1);
            }
            Some(remaining_words) => remaining_words,
        };
    }

    fn best_guess(&self) -> Word {
        // Optimization: hard code the best first guess for Wordle
        if self.guesses.is_empty() {
            return Word::from_str("SERAI");
        }

        let mut best_score = 1000000000;
        let mut best_guess = Word::from_str("_____");
        for guess in &self.allowed_guesses {
            let mut partition = Partition::new();
            for solution in &self.remaining_words {
                let coloring = color(*guess, *solution);
                partition.insert(*solution, coloring);
            }
            let score = partition.worst_case_score();
            if score < best_score {
                best_score = score;
                best_guess = *guess;
            }
        }
        best_guess
    }
}

fn read_input() -> String {
    use std::io::Write;

    print!("> ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input == "" || input == "q" || input == "Q" || input == "quit" || input == "Quit" {
        println!("Have a nice day.");
        std::process::exit(0);
    }

    input.to_string()
}

fn main() {
    use colored::Colorize as _;

    let mut game = GameState::new();

    println!("Hello, wordle!");
    loop {
        println!();
        if !game.guesses.is_empty() {
            println!("════════════════");
            println!("The game so far:");
            println!("  ┌─────┐");
            for (guess, coloring) in &game.guesses {
                println!("  │{}│", guess.to_terminal_colored_string(*coloring));
            }
            println!("  └─────┘");
        }
        match game.remaining_words.len() {
            1 => {
                println!("The solution is {}!", game.remaining_words.remove(0));
                std::process::exit(0);
            }
            2 | 3 | 4 | 5 => {
                print!("Possible words:");
                for word in &game.remaining_words {
                    print!(" {}", word);
                }
                println!();
            }
            n => {
                println!("Possible words: {}", n);
            }
        }
        let guess = game.best_guess();
        println!("Best guess: {}", guess.to_string().bold().purple());
        println!();
        println!("What coloring did that guess get? Use a form like 'wwgyw',");
        println!("or say 'q' or 'quit' or nothing at all to exit.");
        println!();
        let input = read_input();
        let coloring = match Coloring::from_str(&input) {
            None => {
                println!("That's not a valid coloring. Let's try that again.");
                continue;
            }
            Some(coloring) => coloring,
        };
        game.insert_feedback(guess, coloring);
    }
}
