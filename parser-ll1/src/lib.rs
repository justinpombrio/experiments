// TODO: temporary
#![allow(unused)]

pub mod lexer;

use lexer::{LexemeIter, LexerBuilder, Token};
use regex::Error as RegexError;
use std::cell::OnceCell;
use std::iter::Peekable;
use std::ops::{Add, BitOr};
use std::rc::{Rc, Weak};
use std::slice;
use thiserror::Error;

/*========================================*/
/*               Parsing                  */
/*========================================*/

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

pub struct Parser<P: Parse> {
    empty: bool,
    initial: Vec<Token>,
    parse: P,
}

pub trait Parse {
    type Output;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError>;
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    CustomError(String),
    #[error("Parse error: expected {expected} but found {found}")]
    WrongToken { expected: String, found: String },
    #[error("Parse error: expected {expected} but found end of file")]
    NoToken { expected: String },
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

impl<P: Parse> Parser<P> {
    pub fn map<R>(
        self,
        func: impl Fn(P::Output) -> R,
    ) -> Result<Parser<impl Parse<Output = R>>, GrammarError> {
        Ok(MapP::new(self, func))
    }

    pub fn try_map<R>(
        self,
        func: impl Fn(P::Output) -> Result<R, String>,
    ) -> Result<Parser<impl Parse<Output = R>>, GrammarError> {
        Ok(TryMapP::new(self, func))
    }

    pub fn seq2<P1: Parse>(
        self,
        parser_1: Parser<P1>,
    ) -> Result<Parser<impl Parse<Output = (P::Output, P1::Output)>>, GrammarError> {
        Seq2::new(self, parser_1)
    }
}

/*========================================*/
/*               Grammar                  */
/*========================================*/

pub struct Grammar {
    lexer_builder: LexerBuilder,
}

#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("Invalid regex")]
    RegexError(#[from] RegexError),
    #[error("Ambiguous grammar: unclear which choice to take on empty input.")]
    AmbiguityErrorEmpty,
    #[error("Ambiguous grammar: unclear which choice to take on input `{0}`.")]
    AmbiguityErrorFirstToken(String),
}

impl Grammar {
    pub fn new() -> Grammar {
        let lexer_builder = LexerBuilder::new(" \t\n\r")
            .map_err(GrammarError::RegexError)
            .expect("Bug: default whitespace regex");
        Grammar { lexer_builder }
    }

    pub fn new_with_whitespace(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar { lexer_builder })
    }

    pub fn string(
        &mut self,
        string: &str,
    ) -> Result<Parser<impl Parse<Output = ()>>, GrammarError> {
        StringP::new(&mut self.lexer_builder, string)
    }

    pub fn regex<R, F: Fn(&str) -> R>(
        &mut self,
        label: &str,
        pattern: &str,
        func: F,
    ) -> Result<Parser<impl Parse<Output = R>>, GrammarError> {
        RegexP::new(&mut self.lexer_builder, label, pattern, func)
    }
}

/*========================================*/
/*           Parser: String               */
/*========================================*/

struct StringP {
    token: Token,
    string: String,
}

impl StringP {
    fn new(
        lexer_builder: &mut LexerBuilder,
        string: &str,
    ) -> Result<Parser<StringP>, GrammarError> {
        let token = lexer_builder
            .string(string)
            .map_err(GrammarError::RegexError)?;
        Ok(Parser {
            empty: false,
            initial: vec![token],
            parse: StringP {
                token,
                string: string.to_owned(),
            },
        })
    }
}

impl Parse for StringP {
    type Output = ();

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                stream.next();
                return Ok(());
            }
        }
        Err(ParseError::new(
            &self.string,
            stream.next().map(|lex| lex.lexeme),
        ))
    }
}

/*========================================*/
/*           Parser: Regex                */
/*========================================*/

struct RegexP<R, F: Fn(&str) -> R> {
    label: String,
    token: Token,
    pattern: String,
    func: F,
}

impl<R, F: Fn(&str) -> R> RegexP<R, F> {
    fn new(
        lexer_builder: &mut LexerBuilder,
        label: &str,
        pattern: &str,
        func: F,
    ) -> Result<Parser<RegexP<R, F>>, GrammarError> {
        let token = lexer_builder
            .regex(pattern)
            .map_err(GrammarError::RegexError)?;
        Ok(Parser {
            empty: false,
            initial: vec![token],
            parse: RegexP {
                label: label.to_owned(),
                token,
                pattern: pattern.to_owned(),
                func,
            },
        })
    }
}

impl<R, F: Fn(&str) -> R> Parse for RegexP<R, F> {
    type Output = R;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                let result = (self.func)(lexeme.lexeme);
                stream.next();
                return Ok(result);
            }
        }
        Err(ParseError::new(
            &self.label,
            stream.next().map(|lex| lex.lexeme),
        ))
    }
}

/*========================================*/
/*           Parser: Map                  */
/*========================================*/

struct MapP<P: Parse, R, F: Fn(P::Output) -> R> {
    parse: P,
    func: F,
}

impl<P: Parse, R, F: Fn(P::Output) -> R> MapP<P, R, F> {
    fn new(parser: Parser<P>, func: F) -> Parser<MapP<P, R, F>> {
        Parser {
            empty: parser.empty,
            initial: parser.initial,
            parse: MapP {
                parse: parser.parse,
                func,
            },
        }
    }
}

impl<P: Parse, R, F: Fn(P::Output) -> R> Parse for MapP<P, R, F> {
    type Output = R;

    fn parse(&self, stream: &mut TokenStream) -> Result<R, ParseError> {
        Ok((self.func)(self.parse.parse(stream)?))
    }
}

/*========================================*/
/*           Parser: Try Map              */
/*========================================*/

struct TryMapP<P: Parse, R, F: Fn(P::Output) -> Result<R, String>> {
    parse: P,
    func: F,
}

impl<P: Parse, R, F: Fn(P::Output) -> Result<R, String>> TryMapP<P, R, F> {
    fn new(parser: Parser<P>, func: F) -> Parser<TryMapP<P, R, F>> {
        Parser {
            empty: parser.empty,
            initial: parser.initial,
            parse: TryMapP {
                parse: parser.parse,
                func,
            },
        }
    }
}

impl<P: Parse, R, F: Fn(P::Output) -> Result<R, String>> Parse for TryMapP<P, R, F> {
    type Output = R;

    fn parse(&self, stream: &mut TokenStream) -> Result<R, ParseError> {
        match (self.func)(self.parse.parse(stream)?) {
            Ok(result) => Ok(result),
            Err(msg) => Err(ParseError::CustomError(msg)),
        }
    }
}

/*========================================*/
/*           Parser: Recursion            */
/*========================================*/

struct RecurP<P: Parse>(Rc<OnceCell<P>>);

impl<P: Parse> RecurP<P> {
    fn new(make_parser: impl FnOnce(Parser<RecurP<P>>) -> Parser<P>) -> Parser<RecurP<P>> {
        let cell = Rc::new(OnceCell::new());
        let recur = Parser {
            empty: false,        // TODO
            initial: Vec::new(), // TODO!
            parse: RecurP(cell.clone()),
        };
        let parser = make_parser(recur);
        cell.set(parser.parse);
        Parser {
            empty: false,        // TODO
            initial: Vec::new(), // TODO!
            parse: RecurP(cell),
        }
    }
}

impl<P: Parse> Parse for RecurP<P> {
    type Output = P::Output;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        self.0.get().unwrap().parse(stream)
    }
}

/*========================================*/
/*           Parser: Sequencing           */
/*========================================*/

struct Seq2<P0: Parse, P1: Parse>(P0, P1);

impl<P0: Parse, P1: Parse> Seq2<P0, P1> {
    fn new(p0: Parser<P0>, mut p1: Parser<P1>) -> Result<Parser<Seq2<P0, P1>>, GrammarError> {
        let empty = p0.empty && p1.empty;
        let mut initial = p0.initial;
        if p0.empty {
            initial.append(&mut p1.initial);
        }
        Ok(Parser {
            empty,
            initial,
            parse: Seq2(p0.parse, p1.parse),
        })
    }
}

impl<P0: Parse, P1: Parse> Parse for Seq2<P0, P1> {
    type Output = (P0::Output, P1::Output);

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}

/*========================================*/
/*           Parser: Choice               */
/*========================================*/

/*
struct Choice2<P0: Parse, P1: Parse<Output = P0::Output>> {
    parsers: (P0, P1),
    token_to_choice:
}
*/
