use super::source::{Src, Srcloc};
use std::iter::Peekable;
use std::str;
use typed_arena::Arena;

#[derive(Debug)]
pub enum ParseError {
    EmptyFile,
    UnopenedParen,
    UnclosedParen,
    UnclosedString,
    MissingConstruct,
}

pub fn parse_sexpr<'s>(
    arena: &'s Arena<Vec<Src<'s>>>,
    source: &'s str,
) -> Result<Src<'s>, ParseError> {
    let mut parser = SExprParser::new(arena, source);
    parser.parse()
}

struct SExprParser<'s> {
    arena: &'s Arena<Vec<Src<'s>>>,
    chars: Peekable<str::Chars<'s>>,
    remaining: &'s str,
    line: usize,
    column: usize,
}

impl<'s> SExprParser<'s> {
    fn new(arena: &'s Arena<Vec<Src<'s>>>, source: &'s str) -> SExprParser<'s> {
        SExprParser {
            arena,
            chars: source.chars().peekable(),
            remaining: source,
            line: 0,
            column: 0,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    // Panics if there is no next char. Use peek_char() first.
    fn consume_char(&mut self) {
        let ch = self.chars.next().unwrap();
        if ch == '\r' || ch == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        self.remaining = &self.remaining[ch.len_utf8()..];
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => self.consume_char(),
                _ => break,
            }
        }
    }

    fn parse(&mut self) -> Result<Src<'s>, ParseError> {
        self.consume_whitespace();
        match self.peek_char() {
            None => Err(ParseError::EmptyFile),
            Some('(') => Ok(self.parse_sexpr()?),
            Some(')') => Err(ParseError::UnopenedParen),
            Some('"') => Ok(self.parse_string()?),
            Some(ch) => {
                if ch == '.' || ch.is_ascii_digit() {
                    Ok(self.parse_number()?)
                } else {
                    Ok(self.parse_identifier()?)
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<Src<'s>, ParseError> {
        let start_loc = self.srcloc_start();
        self.consume_char(); // skip open quote

        let mut is_escaped = false;
        while let Some(ch) = self.peek_char() {
            self.consume_char();
            match ch {
                '"' if !is_escaped => {
                    let loc = self.srcloc_end(start_loc);
                    return Ok(Src {
                        loc,
                        construct: "string",
                        args: &[],
                    });
                }
                '\\' => {
                    if !is_escaped {
                        is_escaped = true;
                    }
                }
                _ => {
                    is_escaped = false;
                }
            }
        }
        Err(ParseError::UnclosedString)
    }

    fn parse_number(&mut self) -> Result<Src<'s>, ParseError> {
        let start_loc = self.srcloc_start();
        let mut is_float = false;
        while let Some(ch) = self.peek_char() {
            if ch == '.' {
                self.consume_char();
                is_float = true;
            } else if ch.is_ascii_digit() {
                self.consume_char();
            } else {
                break;
            }
        }

        let loc = self.srcloc_end(start_loc);
        let construct = if is_float { "float" } else { "int" };
        Ok(Src {
            loc,
            construct,
            args: &[],
        })
    }

    fn parse_identifier(&mut self) -> Result<Src<'s>, ParseError> {
        let start_loc = self.srcloc_start();
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => break,
                _ => self.consume_char(),
            }
        }
        let loc = self.srcloc_end(start_loc);
        Ok(Src {
            loc,
            construct: "id",
            args: &[],
        })
    }

    fn parse_sexpr(&mut self) -> Result<Src<'s>, ParseError> {
        let start_loc = self.srcloc_start();
        self.consume_char(); // skip open paren
        self.consume_whitespace();
        let construct = self.parse_identifier()?.loc.source;
        if construct.is_empty() {
            return Err(ParseError::MissingConstruct);
        }
        let args = self.arena.alloc(vec![]);
        loop {
            self.consume_whitespace();
            if let Some(ch) = self.peek_char() {
                if ch == ')' {
                    self.consume_char();
                    let loc = self.srcloc_end(start_loc);
                    return Ok(Src {
                        loc,
                        construct,
                        args,
                    });
                } else {
                    args.push(self.parse()?);
                }
            } else {
                break;
            }
        }
        Err(ParseError::UnclosedParen)
    }

    fn srcloc_start(&self) -> Srcloc<'s> {
        Srcloc {
            line: self.line,
            column: self.column,
            source: self.remaining,
        }
    }

    fn srcloc_end(&self, mut loc: Srcloc<'s>) -> Srcloc<'s> {
        let len = loc.source.len() - self.remaining.len();
        loc.source = &loc.source[..len];
        loc
    }
}

#[test]
fn test_sexpr_parser() {
    let arena = Arena::new();
    let source = "(+ 1\n 2.)";
    let expr = parse_sexpr(&arena, source).unwrap();

    assert_eq!(expr.loc.source, "(+ 1\n 2.)");
    assert_eq!(expr.loc.line, 0);
    assert_eq!(expr.loc.column, 0);
    assert_eq!(expr.construct, "+");
    assert_eq!(expr.args.len(), 2);

    let arg = expr.args[0];
    assert_eq!(arg.loc.source, "1");
    assert_eq!(arg.loc.line, 0);
    assert_eq!(arg.loc.column, 3);
    assert_eq!(arg.construct, "int");
    assert_eq!(arg.args.len(), 0);

    let arg = expr.args[1];
    assert_eq!(arg.loc.source, "2.");
    assert_eq!(arg.loc.line, 1);
    assert_eq!(arg.loc.column, 1);
    assert_eq!(arg.construct, "float");
    assert_eq!(arg.args.len(), 0);

    let arena = Arena::new();
    let source = r#" "\"\ax\\\"""#;
    let expr = parse_sexpr(&arena, source).unwrap();
    assert_eq!(expr.loc.source, r#""\"\ax\\\"""#);
    assert_eq!(expr.loc.line, 0);
    assert_eq!(expr.loc.column, 1);
    assert_eq!(expr.construct, "string");
}
