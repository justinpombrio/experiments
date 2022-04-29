//! An unnecessary abstraction. Don't use me.

//! Compute line and column info for a source file.
//!
//! Upon construction, the `LineAndColIndexer` will scan the source file once for newlines. After
//! construction, you can query for the line&column of a position within the file in O(1) time.
//!
//! ```
//! use lexer::line_and_col_indexer::LineAndColIndexer;
//!
//! //                                    012345 6 7890
//! let counter = LineAndColIndexer::new("abc d\r\n ef\n");
//!
//! // There are two lines
//! assert_eq!(counter.num_lines(), 2);
//!
//! // "c" is at line 0, col 2
//! assert_eq!(counter.line_col(2), (0, 2));
//!
//! // "d" is at line 0, col 4
//! assert_eq!(counter.line_col(4), (0, 4));
//!
//! // "e" is at line 1, col 1
//! assert_eq!(counter.line_col(8), (1, 1));
//!
//! // View all of "e"s line
//! assert_eq!(counter.line_contents(1), " ef");
//! ```

use std::fmt;

/// A store of newline locations within a source text, for the purpose of quickly computing line
/// and column positions.
#[derive(Debug, Clone)]
pub struct LineAndColIndexer<'s> {
    source: &'s str,
    newline_positions: Vec<usize>,
}

pub struct Pos {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl<'s> LineAndColIndexer<'s> {
    /// Construct a line counter for the source file. This will scan the file for newlines, to
    /// allow all further operations to be O(1).
    pub fn new(source: &'s str) -> LineAndColIndexer<'s> {
        let mut pos = 0;
        let mut newline_positions = vec![0];
        for ch in source.chars() {
            pos += ch.len_utf8();
            if ch == '\n' {
                newline_positions.push(pos);
            }
        }
        LineAndColIndexer {
            source,
            newline_positions,
        }
    }

    /// Get the original source text.
    pub fn source(&self) -> &'s str {
        self.source
    }

    /// Get the total number of lines in the source.
    pub fn num_lines(&self) -> usize {
        self.newline_positions.len() - 1
    }

    /// Lookup the position (line&col) of the start of a substring within the source string.
    pub fn start(&self, lexeme: &str) -> Pos {
        let start = self.offset(lexeme);
        let (line, col) = self.line_col(start);
        Pos {
            offset: start,
            line,
            col,
        }
    }

    /// For best-case speed testing
    pub fn start_col(&self, lexeme: &str) -> usize {
        let pos = self.offset(lexeme);
        let line = match self.newline_positions.binary_search(&pos) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        pos - self.newline_positions[line]
    }

    /// For best-case speed testing
    pub fn end_utf8_col(&self, lexeme: &str) -> usize {
        let pos = self.offset(lexeme) + lexeme.len();
        let line = match self.newline_positions.binary_search(&pos) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        self.source[self.newline_positions[line]..pos]
            .chars()
            .count()
    }

    /// Lookup the position (line&col) of the end of a substring within the source string.
    pub fn end(&self, lexeme: &str) -> Pos {
        let start = self.offset(lexeme);
        let end = start + lexeme.len();
        let (line, col) = self.line_col(end);
        Pos {
            offset: end,
            line,
            col,
        }
    }

    pub fn offset(&self, lexeme: &str) -> usize {
        let source_ptr = self.source as *const str as *const u8 as usize;
        let start_ptr = lexeme as *const str as *const u8 as usize;
        start_ptr - source_ptr
    }

    /// Get the line and column of a position (byte index) within the source. `pos` is relative to
    /// the start of the `source` string. A newline or return character is considered part of the
    /// line it ends.
    ///
    /// The line and column are 0-indexed. There is unfortunately not a strong consensus on whether
    /// lines should be 0-indexed or 1-indexed; you may need to convert depending on your use case.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is past the end of the source string.
    pub fn line_col(&self, pos: usize) -> (usize, usize) {
        let line = match self.newline_positions.binary_search(&pos) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        let start_pos = self.newline_positions[line];
        let col = pos - start_pos;
        (line, col)
    }

    /// Get the contents of the `line_no`th line. Excludes the line termination character(s).
    ///
    /// # Panics
    ///
    /// Panics if there are fewer than `line_no` lines.
    pub fn line_contents(&self, line_no: usize) -> &'s str {
        let (start, end) = self.line_span(line_no);
        &self.source[start..end]
    }

    /// Like `line_contents`, but includes the line termination character(s).
    pub fn line_contents_inclusive(&self, line_no: usize) -> &'s str {
        let (start, end) = self.line_span_inclusive(line_no);
        &self.source[start..end]
    }

    /// Get the start and end position (byte index) of the `line_no`th line. The start is
    /// inclusive, and the end is exclusive. Does not include the line termination character(s).
    ///
    /// # Panics
    ///
    /// Panics if there are fewer than `line_no` lines.
    pub fn line_span(&self, line_no: usize) -> (usize, usize) {
        let (start, mut end) = self.line_span_inclusive(line_no);
        let line = &self.source[start..end];
        if line.ends_with("\r\n") {
            end -= 2;
        } else if line.ends_with("\n") {
            end -= 1;
        }
        (start, end)
    }

    /// Like `line_span`, but includes the line termination character(s).
    pub fn line_span_inclusive(&self, line_no: usize) -> (usize, usize) {
        let start = self.newline_positions[line_no];
        let end = match self.newline_positions.get(line_no + 1) {
            Some(end_pos) => *end_pos,
            None => self.source.len(),
        };
        (start, end)
    }
}
