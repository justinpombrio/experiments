// TODO: temporary
#![allow(unused)]

mod initial_set;
mod lexer;
mod vec_map;

use initial_set::{ChoiceTable, InitialSet};
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
    initial_set: InitialSet,
    parse: ParseFn<T>,
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

/*========================================*/
/*               Grammar                  */
/*========================================*/

#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("Invalid regex")]
    RegexError(#[from] RegexError),
    #[error("Ambiguous grammar: unclear which choice to take on empty input for {0}.")]
    AmbiguityOnEmpty(String),
    #[error("Ambiguous grammar: unclear which choice to take on input `{2}` for {0}.")]
    AmbiguityOnFirstToken(String, Token, String),
}

pub struct Grammar {
    lexer_builder: LexerBuilder,
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

        let string_copy = string.to_owned();
        let parse = Box::new(move |stream: &mut TokenStream| {
            if let Some(lexeme) = stream.peek() {
                if lexeme.token == token {
                    stream.next();
                    return Ok(());
                }
            }
            Err(ParseError::new(
                &string_copy,
                stream.next().map(|lex| lex.lexeme),
            ))
        });

        Ok(Parser {
            initial_set: InitialSet::new_singleton(token, string.to_owned()),
            parse,
        })
    }

    pub fn regex<T>(
        &mut self,
        pattern: &str,
        func: impl Fn(&str) -> T + 'static,
    ) -> Result<Parser<T>, GrammarError> {
        let token = self
            .lexer_builder
            .regex(pattern)
            .map_err(GrammarError::RegexError)?;

        let pattern_copy = pattern.to_owned();
        let parse = Box::new(move |stream: &mut TokenStream| {
            if let Some(lexeme) = stream.peek() {
                if lexeme.token == token {
                    let result = func(lexeme.lexeme);
                    stream.next();
                    return Ok(result);
                }
            }
            Err(ParseError::new(
                &pattern_copy,
                stream.next().map(|lex| lex.lexeme),
            ))
        });

        Ok(Parser {
            initial_set: InitialSet::new_singleton(token, pattern.to_owned()),
            parse,
        })
    }
}

/*========================================*/
/*               Parsers                  */
/*========================================*/

impl<T: 'static> Parser<T> {
    pub fn map<U>(self, func: impl Fn(T) -> U + 'static) -> Result<Parser<U>, GrammarError> {
        Ok(Parser {
            initial_set: self.initial_set,
            parse: Box::new(move |stream: &mut TokenStream| Ok(func((self.parse)(stream)?))),
        })
    }

    pub fn try_map<U: 'static>(
        self,
        func: impl Fn(T) -> Result<U, String> + 'static,
    ) -> Result<Parser<U>, GrammarError> {
        Ok(Parser {
            initial_set: self.initial_set,
            parse: Box::new(
                move |stream: &mut TokenStream| match func((self.parse)(stream)?) {
                    Ok(result) => Ok(result),
                    Err(msg) => Err(ParseError::CustomError(msg)),
                },
            ),
        })
    }

    pub fn seq2<T0: 'static, T1: 'static>(
        label: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
    ) -> Result<Parser<(T0, T1)>, GrammarError> {
        Ok(Parser {
            initial_set: parser_0.initial_set.seq(label, parser_1.initial_set)?,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                Ok((result_0, result_1))
            }),
        })
    }

    pub fn seq3<T0: 'static, T1: 'static, T2: 'static>(
        label: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
    ) -> Result<Parser<(T0, T1, T2)>, GrammarError> {
        Ok(Parser {
            initial_set: parser_0
                .initial_set
                .seq(label, parser_1.initial_set)?
                .seq(label, parser_2.initial_set)?,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                Ok((result_0, result_1, result_2))
            }),
        })
    }

    pub fn seq4<T0: 'static, T1: 'static, T2: 'static, T3: 'static>(
        label: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
    ) -> Result<Parser<(T0, T1, T2, T3)>, GrammarError> {
        Ok(Parser {
            initial_set: parser_0
                .initial_set
                .seq(label, parser_1.initial_set)?
                .seq(label, parser_2.initial_set)?
                .seq(label, parser_3.initial_set)?,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                Ok((result_0, result_1, result_2, result_3))
            }),
        })
    }

    pub fn choice<const N: usize>(
        label: &str,
        parsers: [Parser<T>; N],
    ) -> Result<Parser<T>, GrammarError> {
        let mut initial_sets = Vec::new();
        let mut parse_fns = Vec::new();
        for parser in parsers {
            initial_sets.push(parser.initial_set);
            parse_fns.push(parser.parse);
        }
        let (choice_table, initial_set) = ChoiceTable::new(label, initial_sets)?;

        let label = label.to_owned();
        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let lexeme = stream.peek();
                match choice_table.lookup(lexeme.map(|lex| lex.token)) {
                    None => Err(ParseError::new(&label, lexeme.map(|lex| lex.lexeme))),
                    Some(i) => (parse_fns[i])(stream),
                }
            }),
        })
    }
}
