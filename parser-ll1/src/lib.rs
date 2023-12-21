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

        let mut initial_set = InitialSet::new(string);
        initial_set.add_token(token, string.to_owned());

        let string = string.to_owned();
        let parse = Box::new(move |stream: &mut TokenStream| {
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
        });

        Ok(Parser { initial_set, parse })
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

        let mut initial_set = InitialSet::new(pattern);
        initial_set.add_token(token, pattern.to_owned());

        let pattern = pattern.to_owned();
        let parse = Box::new(move |stream: &mut TokenStream| {
            if let Some(lexeme) = stream.peek() {
                if lexeme.token == token {
                    let result = func(lexeme.lexeme);
                    stream.next();
                    return Ok(result);
                }
            }
            Err(ParseError::new(
                &pattern,
                stream.next().map(|lex| lex.lexeme),
            ))
        });

        Ok(Parser { initial_set, parse })
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

    pub fn empty(name: &str) -> Result<Parser<()>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.add_empty()?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |_stream| Ok(())),
        })
    }

    pub fn opt(self) -> Result<Parser<Option<T>>, GrammarError> {
        let name = self.initial_set.name().to_owned();
        let none_parser = Parser::<Option<T>>::empty(&name)?.map(|()| None)?;
        let some_parser = self.map(|val| Some(val))?;
        Parser::choice(&name, [none_parser, some_parser])
    }

    pub fn seq2<T0: 'static, T1: 'static>(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
    ) -> Result<Parser<(T0, T1)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                Ok((result_0, result_1))
            }),
        })
    }

    pub fn choice<const N: usize>(
        name: &str,
        parsers: [Parser<T>; N],
    ) -> Result<Parser<T>, GrammarError> {
        let mut initial_sets = Vec::new();
        let mut parse_fns = Vec::new();
        for parser in parsers {
            initial_sets.push(parser.initial_set);
            parse_fns.push(parser.parse);
        }
        let (choice_table, initial_set) = ChoiceTable::new(name, initial_sets)?;

        let name = name.to_owned();
        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let lexeme = stream.peek();
                match choice_table.lookup(lexeme.map(|lex| lex.token)) {
                    None => Err(ParseError::new(&name, lexeme.map(|lex| lex.lexeme))),
                    Some(i) => (parse_fns[i])(stream),
                }
            }),
        })
    }
}

/*========================================*/
/*            Long Sequences              */
/*========================================*/

impl<T> Parser<T> {
    pub fn seq3<T0: 'static, T1: 'static, T2: 'static>(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
    ) -> Result<Parser<(T0, T1, T2)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                Ok((result_0, result_1, result_2))
            }),
        })
    }

    pub fn seq4<T0: 'static, T1: 'static, T2: 'static, T3: 'static>(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
    ) -> Result<Parser<(T0, T1, T2, T3)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;
        initial_set.seq(parser_3.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                Ok((result_0, result_1, result_2, result_3))
            }),
        })
    }

    pub fn seq5<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static>(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
        parser_4: Parser<T4>,
    ) -> Result<Parser<(T0, T1, T2, T3, T4)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;
        initial_set.seq(parser_3.initial_set)?;
        initial_set.seq(parser_4.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                let result_4 = (parser_4.parse)(stream)?;
                Ok((result_0, result_1, result_2, result_3, result_4))
            }),
        })
    }

    pub fn seq6<T0: 'static, T1: 'static, T2: 'static, T3: 'static, T4: 'static, T5: 'static>(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
        parser_4: Parser<T4>,
        parser_5: Parser<T5>,
    ) -> Result<Parser<(T0, T1, T2, T3, T4, T5)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;
        initial_set.seq(parser_3.initial_set)?;
        initial_set.seq(parser_4.initial_set)?;
        initial_set.seq(parser_5.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                let result_4 = (parser_4.parse)(stream)?;
                let result_5 = (parser_5.parse)(stream)?;
                Ok((result_0, result_1, result_2, result_3, result_4, result_5))
            }),
        })
    }

    pub fn seq7<
        T0: 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
    >(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
        parser_4: Parser<T4>,
        parser_5: Parser<T5>,
        parser_6: Parser<T6>,
    ) -> Result<Parser<(T0, T1, T2, T3, T4, T5, T6)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;
        initial_set.seq(parser_3.initial_set)?;
        initial_set.seq(parser_4.initial_set)?;
        initial_set.seq(parser_5.initial_set)?;
        initial_set.seq(parser_6.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                let result_4 = (parser_4.parse)(stream)?;
                let result_5 = (parser_5.parse)(stream)?;
                let result_6 = (parser_6.parse)(stream)?;
                Ok((
                    result_0, result_1, result_2, result_3, result_4, result_5, result_6,
                ))
            }),
        })
    }

    pub fn seq8<
        T0: 'static,
        T1: 'static,
        T2: 'static,
        T3: 'static,
        T4: 'static,
        T5: 'static,
        T6: 'static,
        T7: 'static,
    >(
        name: &str,
        parser_0: Parser<T0>,
        parser_1: Parser<T1>,
        parser_2: Parser<T2>,
        parser_3: Parser<T3>,
        parser_4: Parser<T4>,
        parser_5: Parser<T5>,
        parser_6: Parser<T6>,
        parser_7: Parser<T7>,
    ) -> Result<Parser<(T0, T1, T2, T3, T4, T5, T6, T7)>, GrammarError> {
        let mut initial_set = InitialSet::new(name);
        initial_set.seq(parser_0.initial_set)?;
        initial_set.seq(parser_1.initial_set)?;
        initial_set.seq(parser_2.initial_set)?;
        initial_set.seq(parser_3.initial_set)?;
        initial_set.seq(parser_4.initial_set)?;
        initial_set.seq(parser_5.initial_set)?;
        initial_set.seq(parser_6.initial_set)?;
        initial_set.seq(parser_7.initial_set)?;

        Ok(Parser {
            initial_set,
            parse: Box::new(move |stream: &mut TokenStream| {
                let result_0 = (parser_0.parse)(stream)?;
                let result_1 = (parser_1.parse)(stream)?;
                let result_2 = (parser_2.parse)(stream)?;
                let result_3 = (parser_3.parse)(stream)?;
                let result_4 = (parser_4.parse)(stream)?;
                let result_5 = (parser_5.parse)(stream)?;
                let result_6 = (parser_6.parse)(stream)?;
                let result_7 = (parser_7.parse)(stream)?;
                Ok((
                    result_0, result_1, result_2, result_3, result_4, result_5, result_6, result_7,
                ))
            }),
        })
    }
}
