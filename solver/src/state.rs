use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;

const TEXT_BOX_PADDING: usize = 4;
const TEXT_BOX_WIDTH: usize = 60;

/// A state is a mapping from `State::Var` to `State::Value`.
///
/// This trait does not actually contain a state, it just declares the types for `Var` and `Value`.
/// As such, you typically want to implement it as an empty struct: `struct MyState; ... impl State
/// for MyState { ... }`.
pub trait State: fmt::Debug + 'static {
    type Var: fmt::Debug + Hash + Eq + Ord + Clone + 'static;
    type Value: fmt::Debug + Hash + Eq + Ord + Clone + 'static;

    /// Print the state nicely for debugging. None all `Var`s will be present; those that aren't
    /// should be printed as a "blank" (of some sort).
    fn display(f: &mut String, state: &HashMap<Self::Var, Self::Value>) -> fmt::Result {
        use fmt::Write;
        write!(f, "{:#?}", state)
    }
}

pub fn display_states<S: State>(
    f: &mut fmt::Formatter,
    states: Vec<HashMap<S::Var, S::Value>>,
) -> fmt::Result {
    let mut text_box = TextBox::new(TEXT_BOX_WIDTH);
    for state in states {
        let mut string = String::new();
        S::display(&mut string, &state)?;
        text_box.append(string);
    }
    for line in text_box.completed_lines {
        writeln!(f, "{}", line)?;
    }
    for line in text_box.cur_lines {
        writeln!(f, "{}", line)?;
    }
    Ok(())
}

struct TextBox {
    max_width: usize,
    completed_lines: Vec<String>,
    cur_width: usize,
    cur_lines: Vec<String>,
}

impl TextBox {
    fn new(max_width: usize) -> TextBox {
        TextBox {
            max_width,
            completed_lines: Vec::new(),
            cur_width: 0,
            cur_lines: Vec::new(),
        }
    }

    fn print_line(&mut self, row: usize, col: usize, line: &str) {
        while row >= self.cur_lines.len() {
            self.cur_lines.push(String::new());
        }
        let cur_line = self.cur_lines.get_mut(row).unwrap();
        let len = cur_line.chars().count();
        if col > len {
            let missing_len = col - len;
            cur_line.push_str(&format!("{:spaces$}", "", spaces = missing_len));
        }
        cur_line.push_str(line);
        //self.cur_width = self.cur_width.max(cur_line.chars().count());
    }

    fn append(&mut self, state: String) {
        let mut state_width = 0;
        for line in state.lines() {
            state_width = state_width.max(line.chars().count());
        }
        state_width += TEXT_BOX_PADDING;

        if self.cur_width == 0 || self.cur_width + state_width <= self.max_width {
            // It fits on the current lines. Print it there.
            let col = self.cur_width + TEXT_BOX_PADDING;
            for (row, line) in state.lines().enumerate() {
                self.print_line(row, col, line);
            }
            self.cur_width += state_width;
        } else {
            // It doesn't fit. Finish our current lines and start new ones.
            for line in self.cur_lines.drain(..) {
                self.completed_lines.push(line);
            }
            self.completed_lines.push(String::new());
            for state_line in state.lines() {
                let mut line = format!("{:spaces$}", "", spaces = TEXT_BOX_PADDING);
                line.push_str(state_line);
                self.cur_lines.push(line);
            }
            self.cur_width = state_width;
        }
    }
}
