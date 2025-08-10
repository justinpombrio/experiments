use rand::rngs::ThreadRng;
use std::{fmt, fs, io};

const TEAM_NAMES: [&str; 2] = ["\x1b[0;31mTEAM RED \x1b[39m", "\x1b[0;34mTEAM BLUE\x1b[39m"];
const NUM_WORDS: usize = 4;
const NUM_STRIKES: usize = 2;

fn main() {
    let mut rng = rand::rng();
    let words = pick_words(&mut rng, NUM_WORDS, "words.txt");

    let team_a = TEAM_NAMES[0];
    let team_b = TEAM_NAMES[1];

    prompt(|| {
        println!(
            "
You are playing...

\x1b[0;33m/-------------------\\
| !!! DECRYPTIT !!1 |
\\-------------------/\x1b[0m
"
        );
        println!(
            "You are {team_a}

Press ENTER to see your words"
        );
    });

    prompt(|| {
        println!("The words for {team_a} are:\n");
        for word in &words[0] {
            println!("  {}", word);
        }
        println!();
        println!("Write them down, then press ENTER and send the other team over");
    });

    prompt(|| {
        println!(
            "You are {team_b}

Press ENTER to see your words"
        );
    });

    prompt(|| {
        println!("The words for {team_b} are:\n");
        for word in &words[1] {
            println!("  {}", word);
        }
        println!();
        println!("Write them down, then press ENTER");
    });

    let mut game = Game::new(rng, words);
    loop {
        game.turn();
    }
    //println!("gg");
}

struct Game {
    rng: ThreadRng,
    turn: usize,
    strikes: [usize; 2],
    words: [Vec<String>; 2],
}

impl Game {
    fn new(rng: ThreadRng, words: [Vec<String>; 2]) -> Game {
        Game {
            rng,
            turn: 0,
            strikes: [0, 0],
            words,
        }
    }

    fn turn(&mut self) {
        if let Some(winner) = self.winner() {
            prompt(|| {
                println!("{} WINS!\n", TEAM_NAMES[winner]);
                self.show_score();

                for team in [winner, (winner + 1) % 2] {
                    println!("The words for {} were", TEAM_NAMES[team]);
                    for word in &self.words[team] {
                        println!("  {}", word);
                    }
                    println!();
                }
            });

            println!("gg");
            std::process::exit(0);
        }

        let order = WordOrder::new(&mut self.rng);
        prompt(|| {
            self.show_turn();
            self.show_score();
            println!("Clue giver -- press ENTER when you're ready");
        });
        prompt(|| {
            self.show_turn();
            println!("Give clues for the word order:\n\n  {}", order);
        });
        let this_team = TEAM_NAMES[self.turn % 2];
        let other_team = TEAM_NAMES[(self.turn + 1) % 2];
        let other_team_guess = prompt_for_word_order(|| {
            self.show_turn();
            println!("{other_team} -- guess the word order",);
        });
        let this_team_guess = prompt_for_word_order(|| {
            self.show_turn();
            println!("{this_team} -- guess the word order");
        });
        prompt(|| {
            self.show_turn();
            println!("Results:");

            print!("  {this_team_guess} -- guess by {this_team}");
            if this_team_guess != order {
                self.strikes[self.turn % 2] += 1;
                print!(" (strike!)");
            }
            println!();

            print!("  {other_team_guess} -- guess by {other_team}");
            if other_team_guess == order {
                self.strikes[self.turn % 2] += 1;
                print!(" (strike!)");
            }
            println!();

            println!("  {order} -- answer\n");

            self.show_score();
        });
        self.turn += 1;
    }

    fn winner(&self) -> Option<usize> {
        if self.strikes[0] >= NUM_STRIKES {
            Some(1)
        } else if self.strikes[1] >= NUM_STRIKES {
            Some(0)
        } else {
            None
        }
    }

    fn show_turn(&self) {
        println!(
            "\x1b[1mTurn #{} for {}\n\x1b[33m~~~~~~~~~~~~~~~~~~~~~~~~\x1b[0m\n",
            (self.turn / 2) + 1,
            TEAM_NAMES[self.turn % 2]
        );
    }

    fn show_score(&self) {
        println!("Strikes");
        for i in [self.turn, (self.turn + 1) % 2] {
            let strikes = self.strikes[i % 2];
            print!("  {}  ", TEAM_NAMES[i % 2]);
            for j in 0..NUM_STRIKES {
                if j < strikes {
                    print!("X ");
                } else {
                    print!(". ");
                }
            }
            println!();
            //println!("  {}: {}", TEAM_NAMES[i], self.strikes[i]);
        }
        println!();
    }
}

#[derive(PartialEq, Eq)]
struct WordOrder([usize; NUM_WORDS - 1]);

impl WordOrder {
    fn new(rng: &mut ThreadRng) -> WordOrder {
        WordOrder(rand::seq::index::sample_array(rng, NUM_WORDS).unwrap())
    }

    fn parse(input: &str) -> Option<WordOrder> {
        let nums = input
            .split_whitespace()
            .map(|s| {
                let n = s.parse::<usize>().ok()?;
                if n >= 1 { Some(n - 1) } else { None }
            })
            .collect::<Option<Vec<usize>>>()?;
        for i in 0..nums.len() {
            for j in (i + 1)..nums.len() {
                if nums[i] == nums[j] {
                    return None;
                }
            }
        }
        let array = nums.try_into().ok()?;
        Some(WordOrder(array))
    }
}

impl fmt::Display for WordOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.0.iter();
        if let Some(n) = iter.next() {
            write!(f, "{}", n + 1)?;
            for n in iter {
                write!(f, " {}", n + 1)?;
            }
        }
        Ok(())
    }
}

fn prompt<F: FnOnce()>(closure: F) {
    clear_screen();
    closure();
    println!();
    read_input();
}

fn prompt_for_word_order<F: FnOnce()>(closure: F) -> WordOrder {
    use std::io::Write;

    clear_screen();
    closure();
    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();
        let input = read_input();
        if let Some(word_order) = WordOrder::parse(&input) {
            return word_order;
        }
        println!("\nNo dummy, that's not a valid word order");
    }
}

fn clear_screen() {
    for _ in 0..100 {
        println!();
    }
}

fn read_input() -> String {
    let mut garbage = String::new();
    io::stdin().read_line(&mut garbage).expect("failed to io");
    garbage
}

fn pick_words(rng: &mut ThreadRng, num_words: usize, words_filepath: &str) -> [Vec<String>; 2] {
    use rand::seq::SliceRandom;

    let mut dictionary = fs::read_to_string(words_filepath)
        .expect("Failed to open word list")
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    dictionary.shuffle(rng);
    let mut iter = dictionary.into_iter();
    let words_a = (&mut iter).take(num_words).collect::<Vec<_>>();
    let words_b = iter.take(num_words).collect::<Vec<_>>();
    [words_a, words_b]
}
