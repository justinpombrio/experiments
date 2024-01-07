use crate::lexer::Position;
#[cfg(doc)]
use crate::Parser;
use std::fmt;

/*========================================*/
/*          Parse Error Cause             */
/*========================================*/

#[derive(Debug)]
pub enum ParseErrorCause {
    CustomError {
        message: String,
        span: (Position, Position),
    },
    StandardError {
        expected: String,
        found: (Position, Position),
    },
}

impl ParseErrorCause {
    pub fn build_error(self, filename: &str, source: &str) -> ParseError {
        use ParseErrorCause::{CustomError, StandardError};

        let span = match self {
            CustomError { span, .. } => span,
            StandardError { found, .. } => found,
        };
        let line_contents = match source.lines().nth(span.0.line as usize) {
            Some(line) => line.to_owned(),
            None => "".to_owned(),
        };
        let message = match self {
            CustomError { message, .. } => message,
            StandardError { expected, found } => {
                if found.0 == found.1 {
                    format!("expected {} but found end of file", expected)
                } else {
                    let token = &source[found.0.offset..found.1.offset];
                    format!("expected {} but found '{}'", expected, token)
                }
            }
        };
        ParseError {
            message,
            filename: filename.to_owned(),
            line_contents,
            span,
        }
    }
}

/*========================================*/
/*          Parse Error                   */
/*========================================*/

/// An error encountered while parsing.
///
/// There are two kinds of errors:
///
/// - An error because the input didn't match the grammar, saying what was
///   expected and what token was found instead.
/// - A user-written error thrown from a method like [`Parser::try_map`] or
///   [`Parser::try_span`].
#[derive(Debug)]
pub struct ParseError {
    message: String,
    filename: String,
    line_contents: String,
    span: (Position, Position),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Parse error: {}.", self.message)?;
        let (start, end) = self.span;
        if start.line == end.line {
            writeln!(f, "At '{}' line {}.", self.filename, start.line + 1)?;
            writeln!(f)?;
            writeln!(f, "{}", self.line_contents)?;
            for _ in 0..start.utf8_col {
                write!(f, " ")?;
            }
            let len = (end.utf8_col - start.utf8_col).max(1);
            for _ in 0..len {
                write!(f, "^")?;
            }
        } else {
            writeln!(
                f,
                "At '{}' lines {}-{}.",
                self.filename,
                start.line + 1,
                end.line + 1
            )?;
            writeln!(f)?;
            writeln!(f, "{}", self.line_contents)?;
            for _ in 0..start.utf8_col {
                write!(f, " ")?;
            }
            let line_len = self.line_contents.chars().count();
            for _ in 0..(line_len - start.utf8_col as usize) {
                write!(f, "^")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}
