use crate::parse_error::ParseError;
use crate::pos::{Pos, Span};
use regex::Regex;

pub struct Cursor<'a> {
    pub filename: String,
    pub source: &'a str,
    pub pos: Pos,
}

impl<'a> Cursor<'a> {
    #[must_use]
    pub(crate) fn consume_str(&mut self, prefix: &str, delta: Pos) -> bool {
        if self.str().starts_with(prefix) {
            self.pos = self.pos + delta;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub(crate) fn consume_regex(&mut self, regex: &Regex) -> bool {
        if let Some(re_match) = regex.find(self.str()) {
            let delta = Pos::delta(re_match.as_str());
            self.pos = self.pos + delta;
            true
        } else {
            false
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.pos.offset == self.source.len()
    }

    pub(crate) fn error(&self, message: String) -> ParseError {
        ParseError::new(self.filename.clone(), self.source, message, self.pos, None)
    }

    pub(crate) fn error_from(&self, message: String, start: Pos) -> ParseError {
        ParseError::new(
            self.filename.clone(),
            self.source,
            message,
            start,
            Some(self.pos),
        )
    }

    pub(crate) fn substr_from(&self, start: Pos) -> &'a str {
        &self.source[start.offset..self.pos.offset]
    }

    pub(crate) fn span_from(&self, start: Pos) -> Span<'a> {
        Span {
            start,
            end: self.pos,
            substr: &self.source[start.offset..self.pos.offset],
        }
    }

    fn str(&self) -> &str {
        &self.source[self.pos.offset..]
    }
}
