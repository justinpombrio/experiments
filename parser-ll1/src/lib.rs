// TODO: temporary
#![allow(unused)]

mod initial_set;
mod lexer;
mod vec_map;

use crate::lexer::{LexemeIter, Lexer, Token};
use crate::vec_map::VecMap;
use initial_set::{ChoiceTable, InitialSet};
use lexer::LexerBuilder;
use regex::Error as RegexError;
use regex::Regex;
use std::error::Error;
use std::iter::Peekable;
use thiserror::Error;

type TokenStream<'l, 's> = Peekable<LexemeIter<'l, 's>>;

type ParseFn<T> = Box<dyn Fn(&mut TokenStream) -> Result<T, ParseError>>;

/*========================================*/
/*          Interface                     */
/*========================================*/

pub struct Parser<T>(Box<dyn ParserT<T>>);

trait ParserT<T> {
    fn clone_to_box(&self) -> Box<dyn ParserT<T>>;

    fn compile(
        &self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<T>), GrammarError>;
}

impl<T> Clone for Parser<T> {
    fn clone(&self) -> Parser<T> {
        Parser(self.0.clone_to_box())
    }
}

impl<T> Parser<T> {
    pub fn compile(self, whitespace_regex: &str) -> Result<CompiledParser<T>, GrammarError> {
        let mut lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        let (initial_set, parse_fn) = self.0.compile(&mut lexer_builder)?;
        let lexer = lexer_builder.finish()?;
        Ok(CompiledParser { parse_fn, lexer })
    }
}

pub struct CompiledParser<T> {
    parse_fn: ParseFn<T>,
    lexer: Lexer,
}

impl<T> CompiledParser<T> {
    fn parse(&self, input: &str) -> Result<T, ParseError> {
        let mut token_stream = self.lexer.lex(input).peekable();
        (self.parse_fn)(&mut token_stream)
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
/*          Grammar Errors                */
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

/*========================================*/
/*          Parser: String                */
/*========================================*/

impl Parser<()> {
    pub fn string(pattern: &str) -> Parser<()> {
        Parser(Box::new(StringP(pattern.to_owned())))
    }
}

#[derive(Clone)]
struct StringP(String);

impl ParserT<()> for StringP {
    fn clone_to_box(&self) -> Box<dyn ParserT<()>> {
        Box::new(self.clone())
    }

    fn compile(
        &self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<()>), GrammarError> {
        let token = lexer_builder
            .string(&self.0)
            .map_err(GrammarError::RegexError)?;
        // TODO: so many string names
        let name = format!("'{}'", self.0);
        let mut initial_set = InitialSet::new(&name);
        initial_set.add_token(token, name.clone());

        let parse_fn = Box::new(move |stream: &mut TokenStream| {
            if let Some(lexeme) = stream.peek() {
                if lexeme.token == token {
                    stream.next();
                    return Ok(());
                }
            }
            Err(ParseError::new(&name, stream.next().map(|lex| lex.lexeme)))
        });

        Ok((initial_set, parse_fn))
    }
}

/*========================================*/
/*          Parser: Regex                 */
/*========================================*/

impl<T: 'static> Parser<T> {
    pub fn regex(
        regex: &str,
        func: impl Fn(&str) -> Result<T, String> + Clone + 'static,
    ) -> Parser<T> {
        Parser(Box::new(RegexP {
            regex: regex.to_owned(),
            func,
        }))
    }
}

#[derive(Clone)]
struct RegexP<T, F: Fn(&str) -> Result<T, String> + Clone + 'static> {
    regex: String,
    func: F,
}

impl<T: 'static, F: Fn(&str) -> Result<T, String> + Clone + 'static> ParserT<T> for RegexP<T, F> {
    fn clone_to_box(&self) -> Box<dyn ParserT<T>> {
        Box::new(RegexP {
            regex: self.regex.clone(),
            func: self.func.clone(),
        })
    }

    fn compile(
        &self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<T>), GrammarError> {
        let token = lexer_builder
            .regex(&self.regex)
            .map_err(GrammarError::RegexError)?;
        // TODO: so many string names
        let name = format!("/{}/", self.regex);
        let mut initial_set = InitialSet::new(&name);
        initial_set.add_token(token, name.clone());

        let func = self.func.clone();
        let parse_fn = Box::new(move |stream: &mut TokenStream| {
            if let Some(lexeme) = stream.peek() {
                if lexeme.token == token {
                    let lexeme = stream.next().unwrap();
                    return func(lexeme.lexeme).map_err(ParseError::CustomError);
                }
            }
            Err(ParseError::new(&name, stream.next().map(|lex| lex.lexeme)))
        });

        Ok((initial_set, parse_fn))
    }
}

/*========================================*/
/*          Parser: Seq                   */
/*========================================*/

impl<T: 'static, U: 'static> Parser<(T, U)> {
    pub fn seq2(parser_0: Parser<T>, parser_1: Parser<U>) -> Parser<(T, U)> {
        Parser(Box::new(Seq2(parser_0, parser_1)))
    }
}

#[derive(Clone)]
struct Seq2<T: 'static, U: 'static>(Parser<T>, Parser<U>);

impl<T: 'static, U: 'static> ParserT<(T, U)> for Seq2<T, U> {
    fn clone_to_box(&self) -> Box<dyn ParserT<(T, U)>> {
        Box::new(Seq2(self.0.clone(), self.1.clone()))
    }

    fn compile(
        &self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<(T, U)>), GrammarError> {
        let (init_0, parse_0) = self.0 .0.compile(lexer_builder)?;
        let (init_1, parse_1) = self.1 .0.compile(lexer_builder)?;

        let name = init_0.name();
        let mut init = init_0;
        init.seq(init_1)?;

        let parse_fn = Box::new(move |stream: &mut TokenStream| {
            let result_0 = parse_0(stream)?;
            let result_1 = parse_1(stream)?;
            Ok((result_0, result_1))
        });

        Ok((init, parse_fn))
    }
}

/*========================================*/
/*          Parser: Choice                */
/*========================================*/

impl<T: 'static> Parser<T> {
    pub fn choice<const N: usize>(name: &str, parsers: [Parser<T>; N]) -> Parser<T> {
        Parser(Box::new(Choice {
            name: name.to_owned(),
            parsers,
        }))
    }
}

struct Choice<T: 'static, const N: usize> {
    name: String,
    parsers: [Parser<T>; N],
}

impl<T: 'static, const N: usize> ParserT<T> for Choice<T, N> {
    fn clone_to_box(&self) -> Box<dyn ParserT<T>> {
        Box::new(Choice {
            name: self.name.clone(),
            parsers: self.parsers.clone(),
        })
    }

    fn compile(
        &self,
        lexer_builder: &mut LexerBuilder,
    ) -> Result<(InitialSet, ParseFn<T>), GrammarError> {
        let mut initial_sets = Vec::new();
        let mut parse_fns = Vec::new();
        // can be nicer with https://github.com/rust-lang/rust/issues/79711
        for result in self.parsers.iter().map(|p| p.0.compile(lexer_builder)) {
            let (initial_set, parse_fn) = result?;
            initial_sets.push(initial_set);
            parse_fns.push(parse_fn);
        }
        let (choice_table, initial_set) = ChoiceTable::new(&self.name, initial_sets)?;

        let name = self.name.clone();
        let parse_fn = Box::new(move |stream: &mut TokenStream| {
            let lexeme = stream.peek();
            match choice_table.lookup(lexeme.map(|lex| lex.token)) {
                None => Err(ParseError::new(&name, lexeme.map(|lex| lex.lexeme))),
                Some(i) => (parse_fns[i])(stream),
            }
        });

        Ok((initial_set, parse_fn))
    }
}
