// TODO: temporary
#![allow(unused)]

fn foobar() -> Result<(), GrammarError> {
    use std::str::FromStr;

    let mut g = Grammar::new_with_whitespace(" \t\n")?;

    let num1 = g.regex("[0-9]+", |s| u32::from_str(s).map_err(|e| e.to_string()))?;
    let plus = g.string("+")?;
    let num2 = g.regex("[0-9]+", |s| u32::from_str(s).map_err(|e| e.to_string()))?;
    let expr = Parser::seq3("expr", num1, plus, num2)?;

    let c = g.finish()?;
    c.parse(&expr, "2 + 3");
    Ok(())
}

pub mod lexer;

use lexer::{LexemeIter, LexerBuilder, Token, TOKEN_EOS};
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

pub trait Parser {
    type Output;

    fn tokens(&self) -> Vec<(Token, String)>;

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError>;

    fn map<R>(
        self,
        func: impl Fn(Self::Output) -> R,
    ) -> Result<impl Parser<Output = R>, GrammarError>
    where
        Self: Sized,
    {
        Ok(MapP { parser: self, func })
    }

    fn try_map<R>(
        self,
        func: impl Fn(Self::Output) -> Result<R, String>,
    ) -> Result<impl Parser<Output = R>, GrammarError>
    where
        Self: Sized,
    {
        Ok(TryMapP { parser: self, func })
    }

    fn seq2<P1: Parser>(
        self,
        parser_1: P1,
    ) -> Result<impl Parser<Output = (Self::Output, P1::Output)>, GrammarError>
    where
        Self: Sized,
    {
        Ok(Seq2(self, parser_1))
    }
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

    pub fn string(&mut self, string: &str) -> Result<impl Parser<Output = ()>, GrammarError> {
        let token = self
            .lexer_builder
            .string(string)
            .map_err(GrammarError::RegexError)?;
        Ok(StringP {
            token,
            string: string.to_owned(),
        })
    }

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
}

/*========================================*/
/*           Parser: String               */
/*========================================*/

struct StringP {
    token: Token,
    string: String,
}

impl Parser for StringP {
    type Output = ();

    fn tokens(&self) -> Vec<(Token, String)> {
        vec![(self.token, format!("'{}'", self.string))]
    }

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

impl<R, F: Fn(&str) -> R> Parser for RegexP<R, F> {
    type Output = R;

    fn tokens(&self) -> Vec<(Token, String)> {
        vec![(self.token, format!("/{}/", self.pattern))]
    }

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

struct MapP<P: Parser, R, F: Fn(P::Output) -> R> {
    parser: P,
    func: F,
}

impl<P: Parser, R, F: Fn(P::Output) -> R> Parser for MapP<P, R, F> {
    type Output = R;

    fn tokens(&self) -> Vec<(Token, String)> {
        self.parser.tokens()
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<R, ParseError> {
        Ok((self.func)(self.parser.parse(stream)?))
    }
}

/*========================================*/
/*           Parser: Try Map              */
/*========================================*/

struct TryMapP<P: Parser, R, F: Fn(P::Output) -> Result<R, String>> {
    parser: P,
    func: F,
}

impl<P: Parser, R, F: Fn(P::Output) -> Result<R, String>> Parser for TryMapP<P, R, F> {
    type Output = R;

    fn tokens(&self) -> Vec<(Token, String)> {
        self.parser.tokens()
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<R, ParseError> {
        match (self.func)(self.parser.parse(stream)?) {
            Ok(result) => Ok(result),
            Err(msg) => Err(ParseError::CustomError(msg)),
        }
    }
}

/*========================================*/
/*           Parser: Recursion            */
/*========================================*/

struct RecurP<P: Parser>(Rc<OnceCell<P>>);

impl<P: Parser> RecurP<P> {
    fn new(make_parser: impl FnOnce(RecurP<P>) -> P) -> RecurP<P> {
        let cell = Rc::new(OnceCell::new());
        let recur = RecurP(cell.clone());
        let parser = make_parser(recur);
        cell.set(parser);
        RecurP(cell)
    }
}

impl<P: Parser> Parser for RecurP<P> {
    type Output = P::Output;

    fn tokens(&self) -> Vec<(Token, String)> {
        self.0.get().unwrap().tokens()
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        self.0.get().unwrap().parse(stream)
    }
}

/*========================================*/
/*           Parser: Sequencing           */
/*========================================*/

struct Seq2<P0: Parser, P1: Parser>(P0, P1);

impl<P0: Parser, P1: Parser> Parser for Seq2<P0, P1> {
    type Output = (P0::Output, P1::Output);

    fn tokens(&self) -> Vec<(Token, String)> {
        let mut tokens = self.0.tokens();
        if tokens.iter().any(|(tok, _)| *tok == TOKEN_EOS) {
            tokens.append(&mut self.1.tokens());
        }
        tokens
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}

/*========================================*/
/*           Parser: Choice               */
/*========================================*/

struct Choice2<P0: Parser, P1: Parser<Output = P0::Output>> {
    label: String,
    parsers: (P0, P1),
    empty_index: Option<usize>,
    token_indices: Vec<Option<usize>>,
}

impl<P0: Parser, P1: Parser<Output = P0::Output>> Choice2<P0, P1> {
    fn new(label: &str, p0: P0, p1: P1) -> Result<Choice2<P0, P1>, GrammarError> {
        let p0_tokens = p0.tokens();
        let p1_tokens = p1.tokens();

        let p0_empty = p0_tokens.iter().any(|(tok, _)| *tok == TOKEN_EOS);
        let p1_empty = p1_tokens.iter().any(|(tok, _)| *tok == TOKEN_EOS);
        let empty_index = match (p0_empty, p1_empty) {
            (false, false) => None,
            (true, false) => Some(0),
            (false, true) => Some(1),
            (true, true) => return Err(GrammarError::AmbiguityErrorEmpty),
        };

        let mut token_indices = vec![];
        let p0_token_indices = p0_tokens.into_iter().map(|(tok, patt)| (0, tok, patt));
        let p1_token_indices = p1_tokens.into_iter().map(|(tok, patt)| (1, tok, patt));
        for (index, token, pattern) in p0_token_indices.chain(p1_token_indices) {
            if index >= token_indices.len() {
                token_indices.resize(index + 1, None);
            }
            if token_indices[index] != None {
                return Err(GrammarError::AmbiguityErrorFirstToken(pattern));
            }
            token_indices[index] = Some(token);
        }

        Ok(Choice2 {
            label: label.to_owned(),
            parsers: (p0, p1),
            empty_index,
            token_indices,
        })
    }
}

impl<P0: Parser, P1: Parser<Output = P0::Output>> Parser for Choice2<P0, P1> {
    type Output = P0::Output;

    fn tokens(&self) -> Vec<(Token, String)> {
        let mut tokens = self.parsers.0.tokens();
        tokens.append(&mut self.parsers.1.tokens());
        tokens
    }

    fn parse(&self, stream: &mut TokenStream) -> Result<Self::Output, ParseError> {
        if let Some(lex) = stream.peek() {
            match self.token_indices.get(lex.token) {
                Some(Some(0)) => self.parsers.0.parse(stream),
                Some(Some(1)) => self.parsers.1.parse(stream),
                Some(Some(_)) => unreachable!(),
                None | Some(None) => Err(ParseError::new(
                    &self.label,
                    stream.next().map(|lex| lex.lexeme),
                )),
            }
        } else {
            match self.empty_index {
                None => return Err(ParseError::new(&self.label, None)),
                Some(0) => self.parsers.0.parse(stream),
                Some(1) => self.parsers.1.parse(stream),
                Some(_) => unreachable!(),
            }
        }
    }
}
