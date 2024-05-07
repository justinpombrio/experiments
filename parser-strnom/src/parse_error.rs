use crate::pos::Pos;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    filename: String,
    message: String,
    start: Pos,
    end: Option<Pos>,
    line_contents: String,
}

impl ParseError {
    pub(crate) fn new(
        filename: String,
        source: &str,
        message: String,
        start: Pos,
        end: Option<Pos>,
    ) -> ParseError {
        let line_contents = match source.lines().nth(start.line as usize) {
            Some(line) => line.to_owned(),
            None => "".to_owned(),
        };

        ParseError {
            filename,
            message,
            start,
            end,
            line_contents,
        }
    }

    pub(crate) fn map(mut self, func: impl Fn(String) -> String) -> ParseError {
        self.message = func(self.message);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use colored::Colorize;

        let start = self.start;
        let end = self.end.unwrap_or(self.start);
        let line_num = format!("{}", start.line + 1);
        let margin_width = line_num.len();
        let num_carets = if start.line == end.line {
            (end.col - start.col).max(1) as usize
        } else {
            self.line_contents.chars().count() - start.col as usize
        };

        writeln!(
            f,
            "{}{} {}",
            "parse error".red().bold(),
            ":".bold(),
            self.message.bold(),
        )?;
        writeln!(
            f,
            "{:indent$}{} {}:{}:{}",
            "",
            "-->".blue().bold(),
            self.filename,
            start.line + 1,
            start.col + 1,
            indent = margin_width,
        )?;
        writeln!(
            f,
            "{:indent$}{}",
            "",
            "|".blue().bold(),
            indent = margin_width + 1
        )?;
        writeln!(
            f,
            "{} {}{}",
            line_num.blue().bold(),
            "|".blue().bold(),
            self.line_contents,
        )?;
        writeln!(
            f,
            "{:indent$}{}{:start$}{}",
            "",
            "|".blue().bold(),
            "",
            &"^".repeat(num_carets).red().bold(),
            start = start.col as usize,
            indent = margin_width + 1
        )?;
        writeln!(
            f,
            "{:indent$}{}{:start$}{}",
            "",
            "|".blue().bold(),
            "",
            self.message.red().bold(),
            start = start.col as usize,
            indent = margin_width + 1
        )?;
        write!(
            f,
            "{:indent$}{}",
            "",
            "|".blue().bold(),
            indent = margin_width + 1
        )?;
        Ok(())
    }
}

impl std::error::Error for ParseError {}

#[test]
fn test_parse_errors() {
    colored::control::set_override(false);

    let error = ParseError::new(
        "<test>".to_owned(),
        "123\n456\n7@9\n123",
        "bad number".to_owned(),
        Pos::delta("123\n456\n7"),
        None,
    );
    assert_eq!(
        error.to_string(),
        "parse error: bad number
 --> <test>:3:2
  |
3 |7@9
  | ^
  | bad number
  |"
    );

    let error = ParseError::new(
        "<test>".to_owned(),
        "123\n456\n7@9\n123",
        "bad number".to_owned(),
        Pos::delta("123\n456\n"),
        Some(Pos::delta("123\n456\n7@9")),
    );
    assert_eq!(
        error.to_string(),
        "parse error: bad number
 --> <test>:3:1
  |
3 |7@9
  |^^^
  |bad number
  |"
    );

    let error = ParseError::new(
        "<test>".to_owned(),
        "123\n456\n7@9\n123",
        "bad stuff".to_owned(),
        Pos::delta(""),
        Some(Pos::delta("123\n456\n7@9\n123")),
    );
    assert_eq!(
        error.to_string(),
        "parse error: bad stuff
 --> <test>:1:1
  |
1 |123
  |^^^
  |bad stuff
  |"
    );
}
