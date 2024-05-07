use std::fmt;
use std::ops::Add;

pub type Offset = usize;
pub type Line = u32;
pub type Col = u32;

/// A region of the input text, provided by method [`Parser::span`] and friends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'s> {
    /// The input text from `start` to `end`.
    pub substr: &'s str,
    /// The start of the span, just before its first character.
    pub start: Pos,
    /// The end of the span, just after its last character.
    pub end: Pos,
}

/// A position in the input text, _between_ two characters (or at the
/// start or end of a line). For example, "xyz" has 4 possible positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos {
    /// Byte offset from the beginning of the source string.
    pub offset: Offset,
    /// Line number.
    pub line: Line,
    /// Column number, counted in utf8 codepoints.
    pub col: Col,
}

impl Pos {
    pub fn new() -> Pos {
        Pos {
            offset: 0,
            line: 0,
            col: 0,
        }
    }

    pub fn delta(s: &str) -> Pos {
        let mut line = 0;
        let mut col = 0;
        for ch in s.chars() {
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Pos {
            offset: s.len(),
            line,
            col,
        }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self: Pos, other: Pos) -> Pos {
        Pos {
            offset: self.offset + other.offset,
            line: self.line + other.line,
            col: if other.line == 0 {
                other.col
            } else {
                self.col + other.col
            },
        }
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
