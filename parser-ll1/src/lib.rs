// TODO:
// [x] Make something nicer than seq_n and choice_n for users
// [x] Have recur's interface use mutation, and panic on drop
// [x] Have recur validate, but only to depth 2, using an atomic u8
// [x] ParseError: fancy underlines
// [ ] GrammarError: fix message on choice
// [x] Test errors: give line number, better error message
// [ ] Review&test error messages
// [x] Review combinator names
// [ ] Add iterator combinator for streaming parsing?
// [ ] Add context() combinator?
// [ ] Change Parser<Output = T> to Parser<T>
// [ ] Try having parsers lex directly instead of having a separate lexer;
//     see if that dramatically improves the speed.
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

mod initial_set;
mod lexer;
mod parse_error;
mod parser_recur;
mod tuples;
mod vec_map;

use crate::lexer::{LexemeIter, LexerBuilder, Position, Token};
use dyn_clone::{clone_box, DynClone};
use parse_error::ParseErrorCause;
use regex::Error as RegexError;
use std::error;
use std::fmt;
use thiserror::Error;

#[cfg(feature = "flamegraphs")]
use no_nonsense_flamegraphs::span;

/*========================================*/
/*          Interface                     */
/*========================================*/

pub use initial_set::InitialSet;
pub use parse_error::ParseError;
pub use parser_recur::Recursive;
pub use tuples::{choice, tuple, ChoiceTuple, SeqTuple};

pub enum ParseResult<T> {
    Success(T),
    Failure,
    Error(ParseErrorCause),
}

pub trait Parser: DynClone {
    type Output;

    fn name(&self) -> String;
    fn validate(&self) -> Result<InitialSet, GrammarError>;
    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Self::Output>;

    fn try_map<O>(
        self,
        func: impl Fn(Self::Output) -> Result<O, String> + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        TryMapP { parser: self, func }
    }

    fn map<O>(self, func: impl Fn(Self::Output) -> O + Clone) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        MapP { parser: self, func }
    }

    fn constant<O: Clone>(self, value: O) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map(move |_| Ok(value.clone()))
    }

    fn span<O>(self, func: impl Fn(Span) -> O + Clone) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.map_span(move |span, _| func(span))
    }

    fn map_span<O>(
        self,
        func: impl Fn(Span, Self::Output) -> O + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        SpanP { parser: self, func }
    }

    fn try_span<O, E: error::Error>(
        self,
        func: impl Fn(Span) -> Result<O, E> + Clone,
    ) -> impl Parser<Output = O> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, _| func(span))
    }

    fn try_map_span<O, E: error::Error>(
        self,
        func: impl Fn(Span, Self::Output) -> Result<O, E> + Clone,
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

    fn preceded<P: Parser + Clone>(self, other: P) -> impl Parser<Output = P::Output> + Clone
    where
        Self: Clone,
    {
        self.and(other).map(|(_, v1)| v1)
    }

    fn terminated<P: Parser + Clone>(self, other: P) -> impl Parser<Output = Self::Output> + Clone
    where
        Self: Clone,
    {
        self.and(other).map(|(v0, _)| v0)
    }

    fn complete(self) -> impl Parser<Output = Self::Output> + Clone
    where
        Self: Clone,
    {
        CompleteP(self)
    }

    fn opt(self) -> impl Parser<Output = Option<Self::Output>> + Clone
    where
        Self: Clone,
    {
        // TODO: better name
        let name = self.name().to_owned();
        choice(&name, (self.map(Some), empty().map(|_| None)))
    }

    fn many0(self) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
    {
        ManyP(self)
    }

    fn many1(self) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
    {
        // TODO: this could be more efficient!
        self.clone().and(ManyP(self)).map(|(val, mut vec)| {
            vec.insert(0, val);
            vec
        })
    }

    fn many_sep0(self, sep: impl Parser + Clone) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
    {
        SepP { elem: self, sep }
    }

    fn many_sep1(self, sep: impl Parser + Clone) -> impl Parser<Output = Vec<Self::Output>> + Clone
    where
        Self: Clone,
    {
        let sep_elem = sep.and(self.clone()).map(|(_, v)| v);
        self.clone().and(sep_elem.many0()).map(|(last, mut vec)| {
            vec.insert(0, last);
            vec
        })
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
    ) -> Result<impl Fn(&str, &str) -> Result<P::Output, ParseError>, GrammarError> {
        use ParseResult::{Error, Failure, Success};

        let lexer = self.clone().0.finish();
        let parser = parser.complete(); // ensure whole stream is consumed
        parser.validate()?;

        Ok(move |filename: &str, input: &str| {
            let mut lexemes = lexer.lex(input);
            match parser.parse(&mut lexemes) {
                Success(succ) => Ok(succ),
                Failure => panic!("Bug in CompleteP parser"), // CompleteP never returns Failure
                Error(err) => Err(err.build_error(filename, input)),
            }
        })
    }
}

#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("{0}")]
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
        #[cfg(feature = "flamegraphs")]
        span!("Token");

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
/*          Parser: Try Map               */
/*========================================*/

struct TryMapP<P: Parser + Clone, O, F: Fn(P::Output) -> Result<O, String> + Clone> {
    parser: P,
    func: F,
}

impl<P: Parser + Clone, O, F: Fn(P::Output) -> Result<O, String> + Clone> Clone
    for TryMapP<P, O, F>
{
    fn clone(&self) -> TryMapP<P, O, F> {
        TryMapP {
            parser: self.parser.clone(),
            func: self.func.clone(),
        }
    }
}

impl<P: Parser + Clone, O, F: Fn(P::Output) -> Result<O, String> + Clone> Parser
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

        #[cfg(feature = "flamegraphs")]
        span!("Map");

        let mut skipped_whitespace = stream.clone();
        skipped_whitespace.consume_whitespace();
        let start = skipped_whitespace.pos();
        match self.parser.parse(stream) {
            Success(result) => match (self.func)(result) {
                Ok(succ) => ParseResult::Success(succ),
                Err(err) => {
                    let end = stream.pos();
                    ParseResult::Error(ParseErrorCause::CustomError {
                        message: err,
                        span: (start, end),
                    })
                }
            },
            Failure => Failure,
            Error(err) => Error(err),
        }
    }
}

/*========================================*/
/*          Parser: Map                   */
/*========================================*/

struct MapP<P: Parser + Clone, O, F: Fn(P::Output) -> O + Clone> {
    parser: P,
    func: F,
}

impl<P: Parser + Clone, O, F: Fn(P::Output) -> O + Clone> Clone for MapP<P, O, F> {
    fn clone(&self) -> MapP<P, O, F> {
        MapP {
            parser: self.parser.clone(),
            func: self.func.clone(),
        }
    }
}

impl<P: Parser + Clone, O, F: Fn(P::Output) -> O + Clone> Parser for MapP<P, O, F> {
    type Output = O;

    fn name(&self) -> String {
        self.parser.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<O> {
        use ParseResult::{Error, Failure, Success};

        #[cfg(feature = "flamegraphs")]
        span!("Map");

        match self.parser.parse(stream) {
            Success(result) => Success((self.func)(result)),
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

        #[cfg(feature = "flamegraphs")]
        span!("Complete");

        match self.0.parse(stream) {
            Success(succ) => match stream.next() {
                None => Success(succ),
                Some(lex) => Error(ParseErrorCause::StandardError {
                    expected: "end of file".to_owned(),
                    found: (lex.start, lex.end),
                }),
            },
            Failure => Error(ParseErrorCause::StandardError {
                expected: self.0.name().to_owned(),
                found: match stream.peek() {
                    Some(lex) => (lex.start, lex.end),
                    None => (stream.pos(), stream.pos()),
                },
            }),
            Error(err) => Error(err),
        }
    }
}

/*========================================*/
/*          Parser: Try Span              */
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

struct TrySpanP<P, O, E, F>
where
    P: Parser + Clone,
    E: error::Error,
    F: Fn(Span, P::Output) -> Result<O, E> + Clone,
{
    parser: P,
    func: F,
}

impl<P, O, E, F> Clone for TrySpanP<P, O, E, F>
where
    P: Parser + Clone,
    E: error::Error,
    F: Fn(Span, P::Output) -> Result<O, E> + Clone,
{
    fn clone(&self) -> TrySpanP<P, O, E, F> {
        TrySpanP {
            parser: self.parser.clone(),
            func: self.func.clone(),
        }
    }
}

impl<P, O, E, F> Parser for TrySpanP<P, O, E, F>
where
    P: Parser + Clone,
    E: error::Error,
    F: Fn(Span, P::Output) -> Result<O, E> + Clone,
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

        #[cfg(feature = "flamegraphs")]
        span!("TrySpan");

        let mut skipped_whitespace = stream.clone();
        skipped_whitespace.consume_whitespace();
        let start = skipped_whitespace.pos();
        let source = skipped_whitespace.remaining_source();

        let result = match self.parser.parse(stream) {
            Success(succ) => succ,
            Failure => return Failure,
            Error(err) => return Error(err),
        };

        let end = stream.pos();
        let substr = &source[0..end.offset - start.offset];
        let span = Span { substr, start, end };

        match (self.func)(span, result) {
            Ok(succ) => ParseResult::Success(succ),
            Err(err) => ParseResult::Error(ParseErrorCause::CustomError {
                message: format!("{}", err),
                span: (span.start, span.end),
            }),
        }
    }
}

/*========================================*/
/*          Parser: Span                  */
/*========================================*/

struct SpanP<P, O, F>
where
    P: Parser + Clone,
    F: Fn(Span, P::Output) -> O + Clone,
{
    parser: P,
    func: F,
}

impl<P, O, F> Clone for SpanP<P, O, F>
where
    P: Parser + Clone,
    F: Fn(Span, P::Output) -> O + Clone,
{
    fn clone(&self) -> SpanP<P, O, F> {
        SpanP {
            parser: self.parser.clone(),
            func: self.func.clone(),
        }
    }
}

impl<P, O, F> Parser for SpanP<P, O, F>
where
    P: Parser + Clone,
    F: Fn(Span, P::Output) -> O + Clone,
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

        #[cfg(feature = "flamegraphs")]
        span!("Span");

        let mut skipped_whitespace = stream.clone();
        skipped_whitespace.consume_whitespace();
        let start = skipped_whitespace.pos();
        let source = skipped_whitespace.remaining_source();

        let result = match self.parser.parse(stream) {
            Success(succ) => succ,
            Failure => return Failure,
            Error(err) => return Error(err),
        };

        let end = stream.pos();
        let substr = &source[0..end.offset - start.offset];
        let span = Span { substr, start, end };

        ParseResult::Success((self.func)(span, result))
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

        #[cfg(feature = "flamegraphs")]
        span!("Seq");

        let start_pos = stream.pos().offset;
        let result_0 = match self.0.parse(stream) {
            Success(succ) => succ,
            Failure => return Failure,
            Error(err) => return Error(err),
        };
        let consumed = stream.pos().offset != start_pos;
        let result_1 = match self.1.parse(stream) {
            Success(succ) => succ,
            Error(err) => return Error(err),
            Failure => {
                if consumed {
                    return Error(ParseErrorCause::StandardError {
                        expected: self.1.name().to_owned(),
                        found: match stream.peek() {
                            Some(lex) => (lex.start, lex.end),
                            None => (stream.pos(), stream.pos()),
                        },
                    });
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

        #[cfg(feature = "flamegraphs")]
        span!("Choice");

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

        #[cfg(feature = "flamegraphs")]
        span!("Many");

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

/*========================================*/
/*          Parser: Sep                   */
/*========================================*/

#[derive(Clone)]
struct SepP<P: Parser + Clone, Q: Parser + Clone> {
    elem: P,
    sep: Q,
}

impl<P: Parser + Clone, Q: Parser + Clone> Parser for SepP<P, Q> {
    type Output = Vec<P::Output>;

    fn name(&self) -> String {
        self.elem.name()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // Initial set for up to 2 elems is guaranteed to be initial set for any number of elems

        let elem_init = self.elem.validate()?;
        let sep_init = self.sep.validate()?;

        let len_0 = InitialSet::new_empty("nothing");
        let len_1 = elem_init.clone();
        let mut len_2 = elem_init.clone();
        len_2.seq(sep_init)?;
        len_2.seq(elem_init)?;

        let mut init = len_0;
        init.union("sep", len_1)?;
        init.union("sep", len_2)?;
        Ok(init)
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Vec<P::Output>> {
        use ParseResult::{Error, Failure, Success};

        #[cfg(feature = "flamegraphs")]
        span!("Many");

        let mut results = Vec::new();
        match self.elem.parse(stream) {
            Success(succ) => results.push(succ),
            Error(err) => return Error(err),
            Failure => return Success(results),
        }
        loop {
            match self.sep.parse(stream) {
                Success(_succ) => match self.elem.parse(stream) {
                    Success(succ) => results.push(succ),
                    Error(err) => return Error(err),
                    Failure => {
                        return Error(ParseErrorCause::StandardError {
                            expected: self.sep.name().to_owned(),
                            found: match stream.peek() {
                                Some(lex) => (lex.start, lex.end),
                                None => (stream.pos(), stream.pos()),
                            },
                        })
                    }
                },
                Error(err) => return Error(err),
                Failure => return Success(results),
            }
        }
    }
}
