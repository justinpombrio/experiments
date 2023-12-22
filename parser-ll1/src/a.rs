// TODO: temporary
#![allow(unused)]

pub mod lexer;

use lexer::{LexerBuilder, Token};
use regex::Error as RegexError;
use std::iter::Peekable;
use std::ops::{Add, BitOr};

pub enum GrammarError {
    RegexError(RegexError),
}

pub enum ParseError {
    Eos,

    Dummy,
}

pub struct Grammar {
    lexer_builder: LexerBuilder,
}

pub struct Parser2<T> {
    initial: Vec<Token>,
    parser: Box<dyn Parse<Output = T>>,
}

pub struct Parser<P: Parse> {
    initial: Vec<Token>,
    parser: P,
}

pub trait Parse {
    type Output;

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError>;

    fn map<B>(self, func: impl Fn(Self::Output) -> B) -> impl Parse<Output = B>
    where
        Self: Sized,
    {
        Map { parser: self, func }
    }

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
}

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

struct Seq<P: Parse, Q: Parse> {
    parser_1: P,
    parser_2: Q,
}

impl<P: Parse, Q: Parse> Parse for Seq<P, Q> {
    type Output = (P::Output, Q::Output);

    fn parse(
        &self,
        stream: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Self::Output, ParseError> {
        let result_1 = self.parser_1.parse(stream)?;
        let result_2 = self.parser_2.parse(stream)?;
        Ok((result_1, result_2))
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
