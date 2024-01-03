// TODO:
// [ ] Make something nicer than seq_n and choice_n for users
// [x] Have recur's interface use mutation, and panic on drop
// [x] Have recur validate, but only to depth 2, using an atomic u8
// [ ] ParseError: fancy underlines
// [ ] GrammarError: fix message on choice
// [x] Test errors: give line number, better error message
// [ ] Review error messages
// [ ] Review combinator names
// [ ] Docs

// This design achieves all of the following:
//
// - The lexer isn't exposed (i.e. `Token` isn't in the interface).
// - The types of parsers is reasonable if a bit long `impl Parser<Output = T>`.
// - The implementation of recursive parsers doesn't threaten to summon Cthulu.
// - Parsers can be cloned without having the illegal `Box<Trait + Clone>`.
// - Implementing a parser combinator isn't too onerous.
// - `InitialSet`s aren't needlessly cloned (except if you call `make_parse_fn`
//   many times, but whatever).
// - No unnecessary boxing.
//
// Any change to the design is liable to break one of these properties, so if
// considering a change check this list first.

mod boilerplate;
mod initial_set;
mod lexer;
mod parser_recur;
mod vec_map;

use crate::lexer::{LexemeIter, LexerBuilder, Position, Token};
use dyn_clone::{clone_box, DynClone};
use regex::Error as RegexError;
use std::fmt;
use thiserror::Error;

pub use boilerplate::{
    choice_2, choice_3, choice_4, choice_5, choice_6, choice_7, choice_8, seq_2, seq_3, seq_4,
    seq_5, seq_6, seq_7, seq_8,
};

pub use parser_recur::Recursive;

/*========================================*/
/*          Interface                     */
/*========================================*/

pub use initial_set::InitialSet;

pub trait Parser: DynClone {
    type Output;

    fn name(&self) -> String;
    fn validate(&self) -> Result<InitialSet, GrammarError>;
    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Self::Output>;

    fn try_map<O: Clone>(
        self,
        func: impl Fn(Self::Output) -> Result<O, String> + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        TryMapP { parser: self, func }
    }

    fn map<O: Clone>(
        self,
        func: impl Fn(Self::Output) -> O + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map(move |v| Ok(func(v)))
    }

    fn value<O: Clone>(self, value: O) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map(move |_| Ok(value.clone()))
    }

    fn span<O: Clone>(self, func: impl Fn(Span) -> O + Clone) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, _| Ok(func(span)))
    }

    fn map_span<O: Clone>(
        self,
        func: impl Fn(Span, Self::Output) -> O + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, val| Ok(func(span, val)))
    }

    fn try_span<O: Clone>(
        self,
        func: impl Fn(Span) -> Result<O, String> + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, _| func(span))
    }

    fn try_map_span<O: Clone>(
        self,
        func: impl Fn(Span, Self::Output) -> Result<O, String> + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        TrySpanP { parser: self, func }
    }

    fn and<P: Parser + Clone>(
        self,
        other: P,
    ) -> impl Parser<Output = (Self::Output, P::Output)> + Clone
    where
        Self: Clone,
    {
        SeqP(self, other)
    }

    fn and_ignore<P: Parser + Clone>(self, other: P) -> impl Parser<Output = Self::Output> + Clone
    where
        Self: Clone,
        Self::Output: Clone,
    {
        self.and(other).map(|(v0, _)| v0)
    }

    fn complete(self) -> impl Parser<Output = Self::Output> + Clone
    where
        Self: Clone,
    {
        CompleteP(self)
    }

    fn or(
        self,
        name: &str,
        other: impl Parser<Output = Self::Output> + Clone,
    ) -> impl Parser<Output = Self::Output> + Clone
    where
        Self: Clone,
    {
        ChoiceP {
            name: name.to_owned(),
            parser_0: self,
            parser_1: other,
        }
    }

    fn opt(self) -> impl Parser<Output = Option<Self::Output>> + Clone
    where
        Self: Clone,
        Self::Output: Clone,
    {
        let name = self.name().to_owned();
        choice_2(&name, self.map(Some), empty().value(None))
    }

    fn many(self) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
    {
        ManyP(self)
    }

    fn sep(self, sep: impl Parser + Clone) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
        Self::Output: Clone,
    {
        let sep_elem = sep.and(self.clone()).map(|(_, v)| v);
        self.clone()
            .and(sep_elem.many())
            .map(|(last, mut vec)| {
                vec.insert(0, last);
                vec
            })
            .opt()
            .map(|opt| opt.unwrap_or_else(|| Vec::new()))
    }
}

impl<T> Clone for Box<dyn Parser<Output = T>> {
    fn clone(&self) -> Self {
        clone_box(self.as_ref())
    }
}

impl<T> Parser for Box<dyn Parser<Output = T>> {
    type Output = T;

    fn name(&self) -> String {
        self.as_ref().name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.as_ref().validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Self::Output> {
        self.as_ref().parse(stream)
    }
}

/*========================================*/
/*          Parse Errors                  */
/*========================================*/

pub enum ParseResult<T> {
    Success(T),
    Failure,
    Error(ParseError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    CustomError(String),
    #[error("Parse error: expected {expected} but found '{found}'.")]
    WrongToken { expected: String, found: String },
    #[error("Parse error: expected {0} but found end of file.")]
    Incomplete(String),
    #[error("Parse error: expected end of file but found '{0}'.")]
    TooMuch(String),
}

impl ParseError {
    pub fn new(expected: String, found: Option<String>) -> ParseError {
        if let Some(found) = found {
            ParseError::WrongToken { expected, found }
        } else {
            ParseError::Incomplete(expected)
        }
    }
}

/*========================================*/
/*          Grammar                       */
/*========================================*/

#[derive(Debug, Clone)]
pub struct Grammar(LexerBuilder);

/// White space as defined by the Pattern_White_Space Unicode property.
pub const UNICODE_WHITESPACE_REGEX: &str =
    "[\\u0009\\u000A\\u000B\\u000C\\u000D\\u0020\\u0085\\u200E\\u200F\\u2028\\u2029]*";

impl Grammar {
    pub fn new() -> Grammar {
        let lexer_builder = LexerBuilder::new(UNICODE_WHITESPACE_REGEX).unwrap();
        Grammar(lexer_builder)
    }

    pub fn with_whitespace(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar(lexer_builder))
    }

    pub fn string(
        &mut self,
        string: &str,
    ) -> Result<impl Parser<Output = ()> + Clone, GrammarError> {
        let name = format!("'{}'", string);
        let token = self.0.string(string)?;
        Ok(TokenP { name, token })
    }

    pub fn regex(
        &mut self,
        name: &str,
        regex: &str,
    ) -> Result<impl Parser<Output = ()> + Clone, GrammarError> {
        let token = self.0.regex(regex)?;
        let name = name.to_owned();
        Ok(TokenP { name, token })
    }

    pub fn make_parse_fn<P: Parser + Clone>(
        &self,
        parser: P,
    ) -> Result<impl Fn(&str) -> Result<P::Output, ParseError>, GrammarError> {
        use ParseResult::{Error, Failure, Success};

        let lexer = self.clone().0.finish();
        let parser = parser.complete(); // ensure whole stream is consumed
        parser.validate()?;

        Ok(move |input: &str| {
            let mut lexemes = lexer.lex(input);
            match parser.parse(&mut lexemes) {
                Success(succ) => Ok(succ),
                Failure => panic!("Bug in CompleteP parser"), // CompleteP never returns Failure
                Error(err) => Err(err),
            }
        })
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
/*          Parser: Empty                 */
/*========================================*/

#[derive(Clone)]
struct EmptyP;

impl Parser for EmptyP {
    type Output = ();

    fn name(&self) -> String {
        "nothing".to_owned()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_empty("nothing"))
    }

    fn parse(&self, _stream: &mut LexemeIter) -> ParseResult<()> {
        ParseResult::Success(())
    }
}

pub fn empty() -> impl Parser<Output = ()> + Clone {
    EmptyP
}

/*========================================*/
/*          Parser: Token                 */
/*========================================*/

#[derive(Clone)]
struct TokenP {
    name: String,
    token: Token,
}

impl Parser for TokenP {
    type Output = ();

    fn name(&self) -> String {
        self.name.clone()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_token(self.name.clone(), self.token))
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<()> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                stream.next();
                return ParseResult::Success(());
            }
        }
        ParseResult::Failure
    }
}

/*========================================*/
/*          Parser: Map                   */
/*========================================*/

#[derive(Clone)]
struct TryMapP<P: Parser, O: Clone, F: Fn(P::Output) -> Result<O, String> + Clone> {
    parser: P,
    func: F,
}

impl<P: Parser + Clone, O: Clone, F: Fn(P::Output) -> Result<O, String> + Clone> Parser
    for TryMapP<P, O, F>
{
    type Output = O;

    fn name(&self) -> String {
        self.parser.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        use ParseResult::{Error, Failure, Success};

        match self.parser.parse(stream) {
            Success(result) => match (self.func)(result) {
                Ok(succ) => ParseResult::Success(succ),
                Err(err) => ParseResult::Error(ParseError::CustomError(err)),
            },
            Failure => Failure,
            Error(err) => Error(err),
        }
    }
}

/*========================================*/
/*          Parser: Complete              */
/*========================================*/

#[derive(Clone)]
struct CompleteP<P: Parser>(P);

impl<P: Parser + Clone> Parser for CompleteP<P> {
    type Output = P::Output;

    fn name(&self) -> String {
        self.0.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.0.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<P::Output> {
        use ParseResult::{Error, Failure, Success};

        match self.0.parse(stream) {
            Success(succ) => match stream.next() {
                None => Success(succ),
                Some(lex) => Error(ParseError::TooMuch(lex.lexeme.to_owned())),
            },
            Failure => Error(ParseError::new(
                self.0.name().to_owned(),
                stream.peek().map(|lex| lex.lexeme.to_owned()),
            )),
            Error(err) => Error(err),
        }
    }
}

/*========================================*/
/*          Parser: Span                  */
/*========================================*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'s> {
    pub substr: &'s str,
    pub start: Position,
    pub end: Position,
}

impl<'s> fmt::Display for Span<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[derive(Clone)]
struct TrySpanP<P: Parser + Clone, O: Clone, F: Fn(Span, P::Output) -> Result<O, String> + Clone> {
    parser: P,
    func: F,
}

impl<P: Parser + Clone, O: Clone, F: Fn(Span, P::Output) -> Result<O, String> + Clone> Parser
    for TrySpanP<P, O, F>
{
    type Output = O;

    fn name(&self) -> String {
        self.parser.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        use ParseResult::{Error, Failure, Success};

        let zero = stream.pos();
        let source = stream.remaining_source();
        let start = stream.peek().map(|lex| lex.start).unwrap_or(zero);

        let result = match self.parser.parse(stream) {
            Success(succ) => succ,
            Failure => return Failure,
            Error(err) => return Error(err),
        };

        let end = stream.pos();
        let substr = &source[start.pos - zero.pos..end.pos - zero.pos];
        let span = Span { substr, start, end };

        match (self.func)(span, result) {
            Ok(succ) => ParseResult::Success(succ),
            Err(err) => ParseResult::Error(ParseError::CustomError(err)),
        }
    }
}

/*========================================*/
/*          Parser: Seq                   */
/*========================================*/

#[derive(Clone)]
struct SeqP<P0: Parser + Clone, P1: Parser + Clone>(P0, P1);

impl<P0: Parser + Clone, P1: Parser + Clone> Parser for SeqP<P0, P1> {
    type Output = (P0::Output, P1::Output);

    fn name(&self) -> String {
        self.0.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        let mut init_0 = self.0.validate()?;
        let init_1 = self.1.validate()?;
        init_0.seq(init_1)?;
        Ok(init_0)
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<(P0::Output, P1::Output)> {
        use ParseResult::{Error, Failure, Success};

        let start_pos = stream.pos();
        let result_0 = match self.0.parse(stream) {
            Success(succ) => succ,
            Failure => return Failure,
            Error(err) => return Error(err),
        };
        let consumed = stream.pos() != start_pos;
        let result_1 = match self.1.parse(stream) {
            Success(succ) => succ,
            Error(err) => return Error(err),
            Failure => {
                if consumed {
                    return Error(ParseError::new(
                        self.1.name().to_owned(),
                        stream.peek().map(|lex| lex.lexeme.to_owned()),
                    ));
                } else {
                    return Failure;
                }
            }
        };
        ParseResult::Success((result_0, result_1))
    }
}

/*========================================*/
/*          Parser: Choice                */
/*========================================*/

#[derive(Clone)]
struct ChoiceP<P0: Parser + Clone, P1: Parser<Output = P0::Output> + Clone> {
    name: String,
    parser_0: P0,
    parser_1: P1,
}

impl<P0: Parser + Clone, P1: Parser<Output = P0::Output> + Clone> Parser for ChoiceP<P0, P1> {
    type Output = P0::Output;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        let mut init_0 = self.parser_0.validate()?;
        let init_1 = self.parser_1.validate()?;
        // TODO: bad grammar error here
        init_0.union(&self.name, init_1)?;
        Ok(init_0)
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<P0::Output> {
        use ParseResult::{Error, Failure, Success};

        match self.parser_0.parse(stream) {
            Success(succ) => Success(succ),
            Error(err) => Error(err),
            Failure => match self.parser_1.parse(stream) {
                Success(succ) => Success(succ),
                Error(err) => Error(err),
                Failure => Failure,
            },
        }
    }
}

/*========================================*/
/*          Parser: Many                  */
/*========================================*/

#[derive(Clone)]
struct ManyP<P: Parser + Clone>(P);

impl<P: Parser + Clone> Parser for ManyP<P> {
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        self.0.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // If `self.0` accepts empty then this union will produce an error.
        // Otherwise the initial set is simply `self.0`s initial set
        // together with empty.
        let mut init = self.0.validate()?;
        init.union("nothing", InitialSet::new_empty("nothing"))?;
        Ok(init)
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Vec<P::Output>> {
        use ParseResult::{Error, Failure, Success};

        let mut results = Vec::new();
        loop {
            match self.0.parse(stream) {
                Success(succ) => results.push(succ),
                Error(err) => return Error(err),
                Failure => return Success(results),
            }
        }
    }
}
