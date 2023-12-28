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
use std::cell::OnceCell;
use std::error::Error;
use std::iter::Peekable;
use std::rc::{Rc, Weak};
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
// - Saying `Parser<T>` instead of `Parser<Output = T>` forces implementing
//   types to use `PhantomData`.

/*========================================*/
/*          Interface                     */
/*========================================*/

type Lexemes<'l, 's> = Peekable<LexemeIter<'l, 's>>;

pub trait Parser: Clone {
    type Output;

    fn initial_set(&self) -> Result<InitialSet, GrammarError>;

    fn parse(&self, stream: &mut Lexemes) -> Result<Self::Output, ParseError>;

    fn boxed(self) -> BoxedP<Self::Output>
    where
        Self: 'static,
    {
        BoxedP(Box::new(self))
    }

    fn map<R: Clone>(self, func: impl Fn(Self::Output) -> R + Clone) -> impl Parser<Output = R>
    where
        Self::Output: Clone,
    {
        MapP { parser: self, func }
    }
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

    pub fn string(&mut self, pattern: &str) -> Result<impl Parser<Output = ()>, GrammarError> {
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
    ) -> Result<impl Parser<Output = T>, GrammarError> {
        let token = self.0.regex(regex)?;
        Ok(RegexP {
            name: name.to_owned(),
            token,
            func,
        })
    }

    pub fn make_parse_fn<T>(
        &self,
        parser: impl Parser<Output = T>,
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

impl Parser for StringP {
    type Output = ();

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

impl<T: Clone, F: Fn(&str) -> Result<T, String> + Clone> Parser for RegexP<T, F> {
    type Output = T;

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

impl Parser for EmptyP {
    type Output = ();

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_empty("Empty"))
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<(), ParseError> {
        Ok(())
    }
}

fn empty() -> impl Parser<Output = ()> {
    EmptyP
}

/*========================================*/
/*          Parser: Map                   */
/*========================================*/

#[derive(Clone)]
struct MapP<T: Clone, P: Parser<Output = T>, U: Clone, F: Fn(T) -> U + Clone> {
    parser: P,
    func: F,
}

impl<T: Clone, P: Parser<Output = T>, U: Clone, F: Fn(T) -> U + Clone> Parser for MapP<T, P, U, F> {
    type Output = U;

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        self.parser.initial_set()
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<U, ParseError> {
        let result = self.parser.parse(stream)?;
        Ok((self.func)(result))
    }
}

/*========================================*/
/*          Parser: Seq2                  */
/*========================================*/

#[derive(Clone)]
struct Seq2<P0: Parser, P1: Parser>(P0, P1);

impl<P0: Parser, P1: Parser> Parser for Seq2<P0, P1> {
    type Output = (P0::Output, P1::Output);

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        let mut initial_set = self.0.initial_set()?;
        initial_set.seq(self.1.initial_set()?)?;
        Ok(initial_set)
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<Self::Output, ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}

pub fn seq2<P0: Parser, P1: Parser>(
    parser_0: P0,
    parser_1: P1,
) -> Result<impl Parser<Output = (P0::Output, P1::Output)>, GrammarError> {
    let parser = Seq2(parser_0, parser_1);
    parser.initial_set()?;
    Ok(parser)
}

/*========================================*/
/*          Parser: Recur                 */
/*========================================*/

#[derive(Clone)]
pub struct Recur<T>(Rc<OnceCell<BoxedP<T>>>);

impl<T: Clone> Parser for Recur<T> {
    type Output = T;

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        self.0.get().unwrap().initial_set()
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError> {
        self.0.get().unwrap().parse(stream)
    }
}

pub fn recur<T: Clone + 'static>(
    make_parser: impl FnOnce(BoxedP<T>) -> Result<BoxedP<T>, GrammarError>,
) -> Result<impl Parser<Output = T>, GrammarError> {
    let cell = Rc::new(OnceCell::new());
    let recur = Recur(cell.clone());
    let boxed_recur = BoxedP(Box::new(recur));
    let parser = make_parser(boxed_recur.clone())?;
    cell.set(parser);
    Ok(boxed_recur)
}

/*========================================*/
/*          Parser: Boxed                 */
/*========================================*/

// This is horrible. Might not be necessary though?

trait BoxedParser {
    type Output;

    fn boxed_clone(&self) -> Box<dyn BoxedParser<Output = Self::Output>>;

    fn boxed_initial_set(&self) -> Result<InitialSet, GrammarError>;

    fn boxed_parse(&self, stream: &mut Lexemes) -> Result<Self::Output, ParseError>;
}

impl<P: Parser + 'static> BoxedParser for P {
    type Output = P::Output;

    fn boxed_clone(&self) -> Box<dyn BoxedParser<Output = P::Output>> {
        Box::new(self.clone())
    }

    fn boxed_initial_set(&self) -> Result<InitialSet, GrammarError> {
        Parser::initial_set(self)
    }

    fn boxed_parse(&self, stream: &mut Lexemes) -> Result<Self::Output, ParseError> {
        Parser::parse(self, stream)
    }
}

pub struct BoxedP<T>(Box<dyn BoxedParser<Output = T>>);

impl<T> Clone for BoxedP<T> {
    fn clone(&self) -> BoxedP<T> {
        BoxedP(self.0.boxed_clone())
    }
}

impl<T> Parser for BoxedP<T> {
    type Output = T;

    fn initial_set(&self) -> Result<InitialSet, GrammarError> {
        self.0.boxed_initial_set()
    }

    fn parse(&self, stream: &mut Lexemes) -> Result<T, ParseError> {
        self.0.boxed_parse(stream)
    }
}

// TODO: temporary testing!

fn use_recur(g: &mut Grammar) -> Result<impl Parser<Output = ()>, GrammarError> {
    let x = g.string("x")?;
    recur(|more| Ok(seq2(x, more)?.map(|_| ()).boxed()))
}
