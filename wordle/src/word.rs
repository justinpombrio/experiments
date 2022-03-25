use std::fmt;

const SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coloring(pub [Color; SIZE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Word([u8; SIZE]);

impl Word {
    pub fn as_bytes(self) -> [u8; SIZE] {
        self.0
    }

    pub fn from_str(s: &str) -> Word {
        let bytes = s.as_bytes();
        assert_eq!(s.len(), SIZE);
        let mut word = [' ' as u8; SIZE];
        for i in 0..SIZE {
            word[i] = bytes[i];
        }
        Word(word)
    }

    pub fn to_string(self) -> String {
        String::from_utf8_lossy(&self.0).to_string()
    }

    pub fn to_terminal_colored_string(self, coloring: Coloring) -> String {
        use colored::Colorize as _;
        use Color::{Green, White, Yellow};

        let mut string = String::new();
        for (ch, color) in self.0.iter().zip(coloring.0) {
            let letter = (*ch as char).to_string();
            match color {
                White => string = format!("{}{}", string, letter),
                Yellow => string = format!("{}{}", string, letter.yellow()),
                Green => string = format!("{}{}", string, letter.green()),
            };
        }
        string
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for ch in self.0 {
            write!(f, "{}", ch as char)?;
        }
        Ok(())
    }
}

impl Coloring {
    pub fn from_str(s: &str) -> Option<Coloring> {
        use Color::{Green, White, Yellow};

        let bytes = s.as_bytes();
        if s.len() != SIZE {
            return None;
        }
        let mut coloring = [Color::White; SIZE];
        for i in 0..SIZE {
            coloring[i] = match bytes[i] as char {
                'W' | 'w' => White,
                'Y' | 'y' => Yellow,
                'G' | 'g' => Green,
                _ => return None,
            };
        }
        Some(Coloring(coloring))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Yellow,
    Green,
}

pub fn color(guess: Word, solution: Word) -> Coloring {
    let guess = guess.as_bytes();
    let solution = solution.as_bytes();

    let mut coloring = Coloring([Color::White; SIZE]);
    for i in 0..SIZE {
        let letter = guess[i];
        if solution[i] == letter {
            coloring.0[i] = Color::Green;
        } else if solution.contains(&letter) {
            let mut guess_count = 0;
            let mut solution_count = 0;
            for j in 0..SIZE {
                if guess[j] == solution[j] {
                    ()
                } else if j < i && guess[j] == letter {
                    guess_count += 1;
                } else if solution[j] == letter {
                    solution_count += 1;
                }
            }
            if guess_count < solution_count {
                coloring.0[i] = Color::Yellow;
            }
        }
    }
    coloring
}

#[test]
fn test_coloring() {
    use Color::Green as G;
    use Color::White as W;
    use Color::Yellow as Y;

    fn run_color(guess: &str, answer: &str) -> Coloring {
        color(Word::from_str(guess), Word::from_str(answer))
    }

    assert_eq!(run_color("paint", "lolly"), Coloring([W, W, W, W, W]));
    assert_eq!(run_color("plain", "lolly"), Coloring([W, Y, W, W, W]));
    assert_eq!(run_color("golly", "lolly"), Coloring([W, G, G, G, G]));
    assert_eq!(run_color("loose", "lolly"), Coloring([G, G, W, W, W]));
    assert_eq!(run_color("olles", "lolly"), Coloring([Y, Y, G, W, W]));
}
