// TODO: temporary
#![allow(unused)]

pub mod lexer;

use lexer::{LexemeIter, LexerBuilder, Token, TOKEN_EOS};
use regex::Error as RegexError;
use std::cell::OnceCell;
use std::iter::Peekable;
use std::ops::{Add, BitOr};
use std::rc::{Rc, Weak};
use std::slice;
use thiserror::Error;

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

type ParseFn<T> = Box<dyn Fn(&mut TokenStream) -> Result<T, ParseError>>;

pub struct Parser<T> {
    on_token: Vec<Option<ParseFn<T>>>,
    on_empty: Option<ParseFn<T>>,
}

impl<T> Parser<T> {
    fn new() -> Parser<T> {
        Parser {
            on_token: Vec::new(),
            on_empty: None,
        }
    }

    fn add_empty(&mut self, func: ParseFn<T>) -> Result<(), GrammarError> {
        if self.on_empty.is_some() {
            return Err(GrammarError::AmbiguityOnEmpty);
        }
        self.on_empty = Some(func);
        Ok(())
    }

    fn add_token(
        &mut self,
        token: Token,
        pattern: &str,
        func: ParseFn<T>,
    ) -> Result<(), GrammarError> {
        if matches!(self.on_token.get(token), Some(Some(_))) {
            return Err(GrammarError::AmbiguityOnFirstToken(pattern.to_owned()));
        }
        if token >= self.on_token.len() {
            self.on_token.resize_with(token + 1, || None);
        }
        self.on_token[token] = Some(func);
        Ok(())
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<T, ParseError> {
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
        if let Some(lex) = stream.peek() {

        } else {
        }
    }
}

/*========================================*/
/*             Parse Errors               */
/*========================================*/

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

/*========================================*/
/*           Parsing Functions            */
/*========================================*/

fn string_parser(token: Token, string: String) -> ParseFn<()> {
    Box::new(move |stream| {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == token {
                stream.next();
                return Ok(());
            }
        }
        Err(ParseError::new(
            &string,
            stream.next().map(|lex| lex.lexeme),
        ))
    })
}

/*
fn seq_parser<T1, T2>(parser_1: Parser<T1>, parser_2: Parser<T2>) -> ParseFn<(T1, T2)> {
    let mut parser = Parser::new();
    if let Some(empty_fn) = parser_1.on_empty {
        parser.add_empty(Box::new(|stream| {
            let t1 = empty_fn(stream)?;

        }));
    }
}
    token: Token, string: String) -> ParseFn<()> {
    Box::new(move |stream| {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == token {
                stream.next();
                return Ok(());
            }
        }
        Err(ParseError::new(
            &string,
            stream.next().map(|lex| lex.lexeme),
        ))
    })
}
*/


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
    AmbiguityOnEmpty,
    #[error("Ambiguous grammar: unclear which choice to take on input `{0}`.")]
    AmbiguityOnFirstToken(String),
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

    pub fn string(&mut self, string: &str) -> Result<Parser<()>, GrammarError> {
        let token = self
            .lexer_builder
            .string(string)
            .map_err(GrammarError::RegexError)?;
        let mut parser = Parser::new();
        parser.add_token(token, string, string_parser(token, string.to_owned()));
        Ok(parser)
    }

    /*
    pub fn regex<R, F: Fn(&str) -> R>(
        &mut self,
        label: &str,
        pattern: &str,
        func: F,
    ) -> Result<impl Parser<Output = R>, GrammarError> {
        let token = self
            .lexer_builder
            .regex(pattern)
            .map_err(GrammarError::RegexError)?;
        Ok(RegexP {
            label: label.to_owned(),
            token,
            pattern: pattern.to_owned(),
            func,
        })
    }
    */
}
