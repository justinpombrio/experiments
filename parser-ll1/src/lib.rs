// TODO: temporary
#![allow(unused)]

mod initial_set;
mod lexer;
mod vec_map;

use crate::lexer::{LexemeIter, Lexer, LexerBuilder, Token};
use crate::vec_map::VecMap;
use initial_set::{ChoiceTable, InitialSet};
use regex::Error as RegexError;
use regex::Regex;
use std::error::Error;
use std::iter::Peekable;
use std::marker::PhantomData;
use thiserror::Error;

// NOTE to self:
//
// There's a big design space here, but most of it gets unweildy or difficult to
// implement in Rust. For example:
//
// - Storing `initial_set` in a `Parser` struct gives the unweildy type:
//   `string_parser(pattern: &str) -> Parser<impl Parse<type Output = ()>>`
// - You can't have `impl Trait` in a `struct` field.
// - A `Box<dyn Parser>` is difficult to clone. (To do so, the trait needs
//   a method `boxed_clone(&self) -> Box<dyn Parser<T>>`.)

/*========================================*/
/*          Interface                     */
/*========================================*/

type Lexemes<'l, 's> = Peekable<LexemeIter<'l, 's>>;

pub trait Parser<T>: Clone {
    fn initial_set(&self) -> Result<InitialSet, GrammarError>;

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError>;
}

/*========================================*/
/*          Parse Errors                  */
/*========================================*/

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    CustomError(String),
    #[error("Parse error: expected {expected} but found {found}")]
    WrongToken { expected: String, found: String },
    #[error("Parse error: expected {expected} but found end of file")]
    NoToken { expected: String },
    #[error("Parse error: found unexpected {found}")]
    Incomplete { found: String },
}

impl ParseError {
    fn new(expected: &str, found: Option<&str>) -> ParseError {
        match found {
            Some(found) => ParseError::WrongToken {
                expected: expected.to_owned(),
                found: found.to_owned(),
            },
            None => ParseError::NoToken {
                expected: expected.to_owned(),
            },
        }
    }
}

/*========================================*/
/*          Grammar                       */
/*========================================*/

#[derive(Clone)]
pub struct Grammar(LexerBuilder);

impl Grammar {
    pub fn new(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar(lexer_builder))
    }

    pub fn string(&mut self, pattern: &str) -> Result<impl Parser<()>, GrammarError> {
        let token = self.0.string(pattern)?;
        Ok(StringP {
            name: pattern.to_owned(),
            token,
        })
    }

    pub fn regex<T: Clone>(
        &mut self,
        name: &str,
        regex: &str,
        func: impl Fn(&str) -> Result<T, String> + Clone,
    ) -> Result<impl Parser<T>, GrammarError> {
        let token = self.0.regex(regex)?;
        Ok(RegexP {
            name: name.to_owned(),
            token,
            func,
        })
    }

    pub fn make_parse_fn<T>(
        &self,
        parser: impl Parser<T>,
    ) -> impl Fn(&str) -> Result<T, ParseError> + Clone {
        // TODO: ensure whole stream is consumed!
        let lexer = self.clone().0.finish();
        move |input: &str| {
            let mut lexemes = lexer.lex(input).peekable();
            parser.parse(&mut lexemes)
        }
    }
}

#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("Invalid regex")]
    RegexError(#[from] RegexError),
    #[error("Ambiguous grammar: parsing {start} could produce either {case_1} or {case_2}.")]
    AmbiguityOnEmpty {
        start: String,
        case_1: String,
        case_2: String,
    },
    #[error("Ambiguous grammar: encountering {pattern} when parsing {start} could produce either {case_1} or {case_2}.")]
    AmbiguityOnFirstToken {
        start: String,
        case_1: String,
        case_2: String,
        pattern: String,
    },
}

/*========================================*/
/*          Parser: String                */
/*========================================*/

#[derive(Clone)]
struct StringP {
    name: String,
    token: Token,
}

impl Parser<()> for StringP {
    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_token(&self.name, self.token))
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<(), ParseError> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                stream.next();
                return Ok(());
            }
        }
        Err(ParseError::new(
            &self.name,
            stream.next().map(|lex| lex.lexeme),
        ))
    }
}

/*========================================*/
/*          Parser: Regex                 */
/*========================================*/

#[derive(Clone)]
struct RegexP<T: Clone, F: Fn(&str) -> Result<T, String> + Clone> {
    name: String,
    token: Token,
    func: F,
}

impl<T: Clone, F: Fn(&str) -> Result<T, String> + Clone> Parser<T> for RegexP<T, F> {
    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_token(&self.name, self.token))
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                let lexeme = stream.next().unwrap();
                return (self.func)(lexeme.lexeme).map_err(ParseError::CustomError);
            }
        }
        Err(ParseError::new(
            &self.name,
            stream.next().map(|lex| lex.lexeme),
        ))
    }
}

/*========================================*/
/*          Parser: Empty                 */
/*========================================*/

#[derive(Clone)]
struct EmptyP;

impl Parser<()> for EmptyP {
    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_empty("Empty"))
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<(), ParseError> {
        Ok(())
    }
}

fn empty() -> impl Parser<()> {
    EmptyP
}

/*========================================*/
/*          Parser: Boxed                 */
/*========================================*/

// This is horrible. Might not be necessary though?

trait ParserWithBoxedClone<T> {
    fn boxed_clone(&self) -> Box<dyn ParserWithBoxedClone<T>>;

    fn initial_set(&self) -> Result<InitialSet, GrammarError>;

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError>;
}

impl<T, P: Parser<T> + 'static> ParserWithBoxedClone<T> for P {
    fn boxed_clone(&self) -> Box<dyn ParserWithBoxedClone<T>> {
        Box::new(self.clone())
    }

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        Parser::initial_set(self)
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError> {
        Parser::parse(self, stream)
    }
}

struct BoxedP<T>(Box<dyn ParserWithBoxedClone<T>>);

impl<T> Clone for BoxedP<T> {
    fn clone(&self) -> BoxedP<T> {
        BoxedP(self.0.boxed_clone())
    }
}

impl<T> Parser<T> for BoxedP<T> {
    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        ParserWithBoxedClone::initial_set(self.0.as_ref())
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError> {
        ParserWithBoxedClone::parse(self.0.as_ref(), stream)
    }
}
