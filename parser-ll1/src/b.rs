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

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("Invalid regex")]
    RegexError(#[from] RegexError),
    #[error("Ambiguous grammar: unclear which choice to take on empty input.")]
    AmbiguityErrorEmpty,
    #[error("Ambiguous grammar: unclear which choice to take on input `{0}`.")]
    AmbiguityErrorFirstToken(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    CustomError(String),
    #[error("Parse error: expected {expected} but found {found}")]
    Missing { expected: String, found: String },
}

pub struct Grammar {
    lexer_builder: LexerBuilder,
}

pub struct Parser<P: Parse> {
    initial: Vec<Token>,
    parser: P,
}

pub trait Parse {
    type Output;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError>;

    /*
    fn map<B>(self, func: impl Fn(Self::Output) -> B) -> impl Parse<Output = B>
    where
        Self: Sized,
    {
        Map { parser: self, func }
    }
    */

    fn seq<P: Parse>(
        self,
        parser: P,
    ) -> Result<impl Parse<Output = (Self::Output, P::Output)>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Seq {
            parser_1: self,
            parser_2: parser,
        })
    }

    /*
    fn then<P: Parse>(self, parser: P) -> Result<impl Parse<Output = P::Output>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Then {
            parser_1: self,
            parser_2: parser,
        })
    }

    fn or<P: Parse<Output = Self::Output>>(
        self,
        parser: P,
    ) -> Result<impl Parse<Output = Self::Output>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Choice {
            parser_1: self,
            parser_2: parser,
        })
    }
    */
}

struct Recur<P: Parse> {
    cell: Rc<OnceCell<P>>,
}

impl<P: Parse> Recur<P> {
    fn new(make_parser: impl FnOnce(Parser<Recur<P>>) -> Parser<P>) -> Parser<Recur<P>> {
        let cell = Rc::new(OnceCell::new());
        let recur = Parser {
            initial: Vec::new(), // TODO!
            parser: Recur { cell: cell.clone() },
        };
        let parser = make_parser(recur);
        cell.set(parser.parser);
        Parser {
            initial: Vec::new(), // TODO!
            parser: Recur { cell },
        }
    }
}

impl<P: Parse> Parse for Recur<P> {
    type Output = P::Output;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        self.cell.get().unwrap().parse(stream)
    }
}

struct Seq<P: Parse, Q: Parse> {
    parser_1: P,
    parser_2: Q,
}

impl<P: Parse, Q: Parse> Parse for Seq<P, Q> {
    type Output = (P::Output, Q::Output);

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        let result_1 = self.parser_1.parse(stream)?;
        let result_2 = self.parser_2.parse(stream)?;
        Ok((result_1, result_2))
    }
}

/*
impl<P: Parse> Parser<P> {
    fn seq<Q: Parse>(
        self,
        parser: Parser<Q>,
    ) -> Result<Parser<impl Parse<Output = (P::Output, Q::Output)>>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Parser {
            initial: Vec::new(), // TODO!

            parser: Seq {
                parser_1: self.parser,
                parser_2: parser.parser,
            },
        })
    }
}

impl Grammar {
    pub fn new(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar { lexer_builder })
    }

    pub fn value<A: Clone>(&mut self, value: A) -> Result<impl Parse<Output = A>, GrammarError> {
        Ok(Value(value))
    }

    pub fn string_old(&mut self, constant: &str) -> Result<impl Parse<Output = ()>, GrammarError> {
        let token = self
            .lexer_builder
            .string(constant)
            .map_err(GrammarError::RegexError)?;
        Ok(TokenP(token))
    }

    pub fn string(
        &mut self,
        constant: &str,
    ) -> Result<Parser<impl Parse<Output = ()>>, GrammarError> {
        let token = self
            .lexer_builder
            .string(constant)
            .map_err(GrammarError::RegexError)?;
        Ok(Parser {
            initial: vec![token],
            parser: TokenP(token),
        })
    }

    pub fn regex(&mut self, regex: &str) -> Result<impl Parse<Output = ()>, GrammarError> {
        let token = self
            .lexer_builder
            .regex(regex)
            .map_err(GrammarError::RegexError)?;
        Ok(TokenP(token))
    }
}

struct TokenP(Token);

impl Parse for TokenP {
    type Output = ();

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        if stream.peek().copied() == Some(self.0) {
            Err(ParseError::Dummy)
        } else {
            stream.next();
            Ok(())
        }
    }
}

struct Value<A: Clone>(A);

impl<A: Clone> Parse for Value<A> {
    type Output = A;

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        Ok(self.0.clone())
    }
}

struct Map<O, P: Parse, F: Fn(P::Output) -> O> {
    parser: P,
    func: F,
}

impl<O, P: Parse, F: Fn(P::Output) -> O> Parse for Map<O, P, F> {
    type Output = O;

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        let output = self.parser.parse(stream)?;
        Ok((self.func)(output))
    }
}


struct Then<P: Parse, Q: Parse> {
    parser_1: P,
    parser_2: Q,
}

impl<P: Parse, Q: Parse> Parse for Then<P, Q> {
    type Output = Q::Output;

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        self.parser_1.parse(stream)?;
        let result_2 = self.parser_2.parse(stream)?;
        Ok(result_2)
    }
}

struct Choice<P: Parse, Q: Parse> {
    parser_1: P,
    parser_2: Q,
}

impl<P: Parse, Q: Parse<Output = P::Output>> Parse for Choice<P, Q> {
    type Output = P::Output;

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        match self.parser_1.parse(stream) {
            Err(_) => self.parser_2.parse(stream),
            Ok(result) => Ok(result),
        }
    }
}
*/
