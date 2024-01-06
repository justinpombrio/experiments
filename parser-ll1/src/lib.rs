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
// [x] Change Parser<Output = T> to Parser<T>
// [x] Try having parsers lex directly instead of having a separate lexer;
//     see if that dramatically improves the speed. Yes: by 20%.
// [ ] Docs
// [ ] Testing

// NOTE: Current time to parse dummy.json: 6.5 ms

// This design achieves all of the following:
//
// - The lexer isn't exposed (i.e. `Token` isn't in the interface).
// - The types of parsers is reasonable if a bit long `impl Parser<T>`.
// - The implementation of recursive parsers doesn't threaten to summon Cthulu.
// - Parsers can be cloned without having the illegal `Box<Trait + Clone>`.
// - Implementing a parser combinator isn't too onerous.
// - `InitialSet`s aren't needlessly cloned (except if you call `make_parse_fn`
//   many times, but whatever).
// - No unnecessary boxing.
//
// Any change to the design is liable to break one of these properties, so if
// considering a change check this list first.
//
// It's tempting to remove the lexer. Doing so yields a ~20% speedup, but it
// would make the parse error messages worse. Not worth it!

mod initial_set;
mod lexer;
mod parse_error;
mod parser_recur;
mod vec_map;

use crate::lexer::{LexemeIter, LexerBuilder, Position, Token};
use dyn_clone::{clone_box, DynClone};
use parse_error::ParseErrorCause;
use regex::Error as RegexError;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use thiserror::Error;

#[cfg(feature = "flamegraphs")]
use no_nonsense_flamegraphs::span;

/*========================================*/
/*          Interface                     */
/*========================================*/

pub use initial_set::InitialSet;
pub use parse_error::ParseError;
pub use parser_recur::Recursive;

pub enum ParseResult<T> {
    Success(T),
    Failure,
    Error(ParseErrorCause),
}

pub trait Parser<T>: DynClone {
    fn name(&self, is_empty: Option<bool>) -> String;
    fn validate(&self) -> Result<InitialSet, GrammarError>;
    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T>;

    fn try_map<T2>(self, func: impl Fn(T) -> Result<T2, String> + Clone) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        TryMapP {
            parser: self,
            func,
            phantom: PhantomData,
        }
    }

    fn map<T2>(self, func: impl Fn(T) -> T2 + Clone) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        MapP {
            parser: self,
            func,
            phantom: PhantomData,
        }
    }

    fn constant<T2: Clone>(self, value: T2) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.try_map(move |_| Ok(value.clone()))
    }

    fn span<T2>(self, func: impl Fn(Span) -> T2 + Clone) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.map_span(move |span, _| func(span))
    }

    fn map_span<T2>(self, func: impl Fn(Span, T) -> T2 + Clone) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        SpanP {
            parser: self,
            func,
            phantom: PhantomData,
        }
    }

    fn try_span<T2, E: error::Error>(
        self,
        func: impl Fn(Span) -> Result<T2, E> + Clone,
    ) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, _| func(span))
    }

    fn try_map_span<T2, E: error::Error>(
        self,
        func: impl Fn(Span, T) -> Result<T2, E> + Clone,
    ) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        TrySpanP {
            parser: self,
            func,
            phantom: PhantomData,
        }
    }

    fn and<T2, P: Parser<T2> + Clone>(self, other: P) -> impl Parser<(T, T2)> + Clone
    where
        Self: Clone,
    {
        SeqP2 {
            name: "sequence".to_owned(),
            parsers: (self, other),
            phantom: PhantomData,
        }
    }

    fn preceded<T2, P: Parser<T2> + Clone>(self, other: P) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.and(other).map(|(_, v1)| v1)
    }

    fn terminated<P: Parser<T> + Clone>(self, other: P) -> impl Parser<T> + Clone
    where
        Self: Clone,
    {
        self.and(other).map(|(v0, _)| v0)
    }

    fn complete(self) -> impl Parser<T> + Clone
    where
        Self: Clone,
    {
        CompleteP(self, PhantomData)
    }

    fn opt(self) -> impl Parser<Option<T>> + Clone
    where
        Self: Clone,
    {
        OptP(self, PhantomData)
    }

    fn many0(self) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        ManyP(self, PhantomData)
    }

    fn many1(self) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        // TODO: this could be more efficient!
        self.clone().and(self.many0()).map(|(val, mut vec)| {
            vec.insert(0, val);
            vec
        })
    }

    fn many_sep0<T2>(self, sep: impl Parser<T2> + Clone) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        SepP {
            elem: self,
            sep,
            phantom: PhantomData,
        }
    }

    fn many_sep1<T2>(self, sep: impl Parser<T2> + Clone) -> impl Parser<Vec<T>> + Clone
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

fn parser_names<T>(parser: &(impl Parser<T> + Clone)) -> (String, String, String) {
    (
        parser.name(None),
        parser.name(Some(true)),
        parser.name(Some(false)),
    )
}

impl<T> Clone for Box<dyn Parser<T>> {
    fn clone(&self) -> Self {
        clone_box(self.as_ref())
    }
}

impl<T> Parser<T> for Box<dyn Parser<T>> {
    fn name(&self, is_empty: Option<bool>) -> String {
        self.as_ref().name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.as_ref().validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T> {
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

    pub fn string(&mut self, string: &str) -> Result<impl Parser<()> + Clone, GrammarError> {
        let name = format!("'{}'", string);
        let token = self.0.string(string)?;
        Ok(TokenP { name, token })
    }

    pub fn regex(
        &mut self,
        name: &str,
        regex: &str,
    ) -> Result<impl Parser<()> + Clone, GrammarError> {
        let token = self.0.regex(regex)?;
        let name = name.to_owned();
        Ok(TokenP { name, token })
    }

    pub fn make_parse_fn<T2, P: Parser<T2> + Clone>(
        &self,
        parser: P,
    ) -> Result<impl Fn(&str, &str) -> Result<T2, ParseError>, GrammarError> {
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
    #[error("Ambiguous grammar: when parsing {name}, there's an ambiguity between {case_1} and {case_2}.")]
    AmbiguityOnEmpty {
        name: String,
        case_1: String,
        case_2: String,
    },
    #[error("Ambiguous grammar: encountering {token} when parsing {name} could indicate either {case_1} or {case_2}.")]
    AmbiguityOnFirstToken {
        name: String,
        case_1: String,
        case_2: String,
        token: String,
    },
}

/*========================================*/
/*          Parser: Empty                 */
/*========================================*/

#[derive(Clone)]
struct EmptyP;

impl Parser<()> for EmptyP {
    fn name(&self, _is_empty: Option<bool>) -> String {
        "nothing".to_owned()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::new_empty())
    }

    fn parse(&self, _stream: &mut LexemeIter) -> ParseResult<()> {
        ParseResult::Success(())
    }
}

pub fn empty() -> impl Parser<()> + Clone {
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

impl Parser<()> for TokenP {
    fn name(&self, is_empty: Option<bool>) -> String {
        if is_empty == Some(true) {
            format!("empty {}", self.name)
        } else {
            self.name.clone()
        }
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

struct TryMapP<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> Result<T1, String> + Clone> {
    parser: P0,
    func: F,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> Result<T1, String> + Clone> Clone
    for TryMapP<T0, P0, T1, F>
{
    fn clone(&self) -> TryMapP<T0, P0, T1, F> {
        TryMapP {
            parser: self.parser.clone(),
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> Result<T1, String> + Clone> Parser<T1>
    for TryMapP<T0, P0, T1, F>
{
    fn name(&self, is_empty: Option<bool>) -> String {
        self.parser.name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T1> {
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

struct MapP<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> T1 + Clone> {
    parser: P0,
    func: F,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> T1 + Clone> Clone for MapP<T0, P0, T1, F> {
    fn clone(&self) -> MapP<T0, P0, T1, F> {
        MapP {
            parser: self.parser.clone(),
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0: Parser<T0> + Clone, T1, F: Fn(T0) -> T1 + Clone> Parser<T1> for MapP<T0, P0, T1, F> {
    fn name(&self, is_empty: Option<bool>) -> String {
        self.parser.name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T1> {
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

struct CompleteP<T, P: Parser<T> + Clone>(P, PhantomData<T>);

impl<T, P: Parser<T> + Clone> Clone for CompleteP<T, P> {
    fn clone(&self) -> Self {
        CompleteP(self.0.clone(), PhantomData)
    }
}

impl<T, P: Parser<T> + Clone> Parser<T> for CompleteP<T, P> {
    fn name(&self, is_empty: Option<bool>) -> String {
        self.0.name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.0.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T> {
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
                expected: self.0.name(None),
                found: stream.upcoming_span(),
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

struct TrySpanP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(Span, T0) -> Result<T1, E1> + Clone,
{
    parser: P0,
    func: F,
    phantom: PhantomData<(T0, T1, E1)>,
}

impl<T0, P0, T1, E1, F> Clone for TrySpanP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(Span, T0) -> Result<T1, E1> + Clone,
{
    fn clone(&self) -> TrySpanP<T0, P0, T1, E1, F> {
        TrySpanP {
            parser: self.parser.clone(),
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0, T1, E1, F> Parser<T1> for TrySpanP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(Span, T0) -> Result<T1, E1> + Clone,
{
    fn name(&self, is_empty: Option<bool>) -> String {
        self.parser.name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T1> {
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

struct SpanP<T0, P0, T1, F>
where
    P0: Parser<T0> + Clone,
    F: Fn(Span, T0) -> T1 + Clone,
{
    parser: P0,
    func: F,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0, T1, F> Clone for SpanP<T0, P0, T1, F>
where
    P0: Parser<T0> + Clone,
    F: Fn(Span, T0) -> T1 + Clone,
{
    fn clone(&self) -> SpanP<T0, P0, T1, F> {
        SpanP {
            parser: self.parser.clone(),
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0, T1, F> Parser<T1> for SpanP<T0, P0, T1, F>
where
    P0: Parser<T0> + Clone,
    F: Fn(Span, T0) -> T1 + Clone,
{
    fn name(&self, is_empty: Option<bool>) -> String {
        self.parser.name(is_empty)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        self.parser.validate()
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T1> {
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

pub fn tuple<T, S: SeqTuple<T>>(name: &str, tuple: S) -> impl Parser<T> + Clone {
    tuple.make_seq(name.to_owned())
}

pub trait SeqTuple<T> {
    fn make_seq(self, name: String) -> impl Parser<T> + Clone;
}

macro_rules! define_seq {
    ($struct:ident, $( ($idx:tt, $type:ident, $parser:ident) ),*) => {
        struct $struct<$( $type ),*, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            name: String,
            parsers: ($( $parser ),*),
            phantom: PhantomData<($( $type ),*)>,
        }

        impl<$( $type ),*, $( $parser ),*> Clone
        for $struct<$( $type ),*, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn clone(&self) -> Self {
                $struct {
                    name: self.name.clone(),
                    parsers: self.parsers.clone(),
                    phantom: PhantomData,
                }
            }
        }

        impl<$( $type ),*, $( $parser ),*> Parser<($( $type ),*)>
        for $struct<$( $type ),*, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn name(&self, is_empty: Option<bool>) -> String {
                match is_empty {
                    None => self.name.clone(),
                    Some(true) => self.parsers.1.name(Some(true)),
                    Some(false) => self.parsers.0.name(Some(false)),
                }
            }

            fn validate(&self) -> Result<InitialSet, GrammarError> {
                InitialSet::sequence(
                    parser_names(self),
                    vec![$( self.parsers.$idx.validate()? ),*],
                )
            }

            fn parse(&self, stream: &mut LexemeIter) -> ParseResult<($( $type ),*)> {
                use ParseResult::{Error, Failure, Success};

                #[cfg(feature = "flamegraphs")]
                span!("Seq");

                let start_pos = stream.pos().offset;
                let results = ( $(
                    match self.parsers.$idx.parse(stream) {
                        Success(succ) => succ,
                        Error(err) => return Error(err),
                        Failure => if stream.pos().offset != start_pos {
                            return Error(ParseErrorCause::StandardError {
                                expected: self.parsers.$idx.name(None),
                                found: stream.upcoming_span(),
                            })
                        } else {
                            return Failure
                        }
                    }
                ),* );
                ParseResult::Success(results)
            }
        }

        impl<$( $type ),*, $( $parser ),*> SeqTuple<($( $type ),*)> for ($( $parser ),*)
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn make_seq(self, name: String) -> impl Parser<($( $type ),*)> + Clone {
                $struct {
                    name,
                    parsers: self,
                    phantom: PhantomData,
                }
            }
        }
    }
}

define_seq!(SeqP2, (0, T0, P0), (1, T1, P1));
define_seq!(SeqP3, (0, T0, P0), (1, T1, P1), (2, T2, P2));
define_seq!(SeqP4, (0, T0, P0), (1, T1, P1), (2, T2, P2), (3, T3, P3));
define_seq!(
    SeqP5,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4)
);
define_seq!(
    SeqP6,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4),
    (5, T5, P5)
);
define_seq!(
    SeqP7,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4),
    (5, T5, P5),
    (6, T6, P6)
);
define_seq!(
    SeqP8,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4),
    (5, T5, P5),
    (6, T6, P6),
    (7, T7, P7)
);
define_seq!(
    SeqP9,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4),
    (5, T5, P5),
    (6, T6, P6),
    (7, T7, P7),
    (8, T8, P8)
);
define_seq!(
    SeqP10,
    (0, T0, P0),
    (1, T1, P1),
    (2, T2, P2),
    (3, T3, P3),
    (4, T4, P4),
    (5, T5, P5),
    (6, T6, P6),
    (7, T7, P7),
    (8, T8, P8),
    (9, T9, P9)
);

/*========================================*/
/*          Parser: Choice                */
/*========================================*/

pub fn choice<T, C: ChoiceTuple<T>>(name: &str, tuple: C) -> impl Parser<T> + Clone {
    tuple.make_choice(name.to_owned())
}

pub trait ChoiceTuple<T> {
    fn make_choice(self, name: String) -> impl Parser<T> + Clone;
}

macro_rules! define_choice {
    ($struct:ident, $type:ident, $( ($idx:tt, $parser:ident) ),*) => {
        struct $struct<$type, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            name: String,
            parsers: ($( $parser ),*),
            phantom: PhantomData<$type>,
        }

        impl<$type, $( $parser ),*> Clone
        for $struct<$type, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn clone(&self) -> Self {
                $struct {
                    name: self.name.clone(),
                    parsers: self.parsers.clone(),
                    phantom: PhantomData,
                }
            }
        }

        impl<$type, $( $parser ),*> Parser<$type>
        for $struct<$type, $( $parser ),*>
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn name(&self, is_empty: Option<bool>) -> String {
                if is_empty == Some(true) {
                    format!("empty {}", self.name)
                } else {
                    self.name.clone()
                }
            }

            fn validate(&self) -> Result<InitialSet, GrammarError> {
                InitialSet::choice(
                    parser_names(self),
                    vec![$( self.parsers.$idx.validate()? ),*],
                )
            }

            fn parse(&self, stream: &mut LexemeIter) -> ParseResult<$type> {
                use ParseResult::{Error, Failure, Success};

                #[cfg(feature = "flamegraphs")]
                span!("Choice");

                $(
                    match self.parsers.$idx.parse(stream) {
                        Success(succ) => return Success(succ),
                        Error(err) => return Error(err),
                        Failure => (),
                    }
                )*
                Failure
            }
        }

        impl<$type, $( $parser ),*> ChoiceTuple<$type> for ($( $parser ),*)
        where $( $parser: Parser<$type> + Clone ),*
        {
            fn make_choice(self, name: String) -> impl Parser<$type> + Clone {
                $struct {
                    name,
                    parsers: self,
                    phantom: PhantomData,
                }
            }
        }
    }
}

define_choice!(ChoiceP2, T, (0, P0), (1, P1));
define_choice!(ChoiceP3, T, (0, P0), (1, P1), (2, P2));
define_choice!(ChoiceP4, T, (0, P0), (1, P1), (2, P2), (3, P3));
define_choice!(ChoiceP5, T, (0, P0), (1, P1), (2, P2), (3, P3), (4, P4));
define_choice!(
    ChoiceP6,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5)
);
define_choice!(
    ChoiceP7,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5),
    (6, P6)
);
define_choice!(
    ChoiceP8,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5),
    (6, P6),
    (7, P7)
);
define_choice!(
    ChoiceP9,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5),
    (6, P6),
    (7, P7),
    (8, P8)
);
define_choice!(
    ChoiceP10,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5),
    (6, P6),
    (7, P7),
    (8, P8),
    (9, P9)
);

/*========================================*/
/*          Parser: Optional              */
/*========================================*/

struct OptP<T, P: Parser<T> + Clone>(P, PhantomData<T>);

impl<T, P: Parser<T> + Clone> Clone for OptP<T, P> {
    fn clone(&self) -> Self {
        OptP(self.0.clone(), PhantomData)
    }
}

impl<T, P: Parser<T> + Clone> Parser<Option<T>> for OptP<T, P> {
    fn name(&self, is_empty: Option<bool>) -> String {
        match is_empty {
            None => format!("optional {}", self.0.name(None)),
            Some(true) => format!("empty optional {}", self.0.name(None)),
            Some(false) => self.0.name(Some(false)),
        }
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // If `self.0` accepts empty then this union will produce an error.
        // Otherwise the initial set is simply `self.0`s initial set
        // together with empty.
        InitialSet::choice(
            parser_names(self),
            vec![InitialSet::new_empty(), self.0.validate()?],
        )
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Option<T>> {
        use ParseResult::{Error, Failure, Success};

        #[cfg(feature = "flamegraphs")]
        span!("Opt");

        match self.0.parse(stream) {
            Success(succ) => Success(Some(succ)),
            Error(err) => Error(err),
            Failure => Success(None),
        }
    }
}

/*========================================*/
/*          Parser: Many                  */
/*========================================*/

struct ManyP<T, P: Parser<T> + Clone>(P, PhantomData<T>);

impl<T, P: Parser<T> + Clone> Clone for ManyP<T, P> {
    fn clone(&self) -> Self {
        ManyP(self.0.clone(), PhantomData)
    }
}

impl<T, P: Parser<T> + Clone> Parser<Vec<T>> for ManyP<T, P> {
    fn name(&self, is_empty: Option<bool>) -> String {
        match is_empty {
            None => format!("many0 of {}", self.0.name(None)),
            Some(true) => format!("empty many0 of {}", self.0.name(None)),
            Some(false) => self.0.name(Some(false)),
        }
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // If `self.0` accepts empty then this union will produce an error.
        // Otherwise the initial set is simply `self.0`s initial set
        // together with empty.
        InitialSet::choice(
            parser_names(self),
            vec![InitialSet::new_empty(), self.0.validate()?],
        )
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Vec<T>> {
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

struct SepP<T0, P0, T1, P1>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
{
    elem: P0,
    sep: P1,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0, T1, P1> Clone for SepP<T0, P0, T1, P1>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
{
    fn clone(&self) -> Self {
        SepP {
            elem: self.elem.clone(),
            sep: self.sep.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0, T1, P1> Parser<Vec<T0>> for SepP<T0, P0, T1, P1>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
{
    fn name(&self, is_empty: Option<bool>) -> String {
        let elem_name = self.elem.name(None);
        let sep_name = self.elem.name(None);
        if is_empty == Some(true) {
            format!("empty {} separated by {}", elem_name, sep_name)
        } else {
            format!("empty {} separated by {}", elem_name, sep_name)
        }
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        let elem_init = self.elem.validate()?;
        let sep_init = self.sep.validate()?;

        // SepBy(E, S) = (.|E(SE)*) ~= (.|E(.|SE))
        let names = parser_names(self);
        let sep_elem = InitialSet::sequence(names.clone(), vec![sep_init, elem_init.clone()])?;
        let tail = InitialSet::choice(parser_names(self), vec![InitialSet::new_empty(), sep_elem])?;
        let nonempty = InitialSet::sequence(parser_names(self), vec![elem_init, tail])?;
        InitialSet::choice(parser_names(self), vec![InitialSet::new_empty(), nonempty])
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<Vec<T0>> {
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
                            expected: self.sep.name(None),
                            found: stream.upcoming_span(),
                        })
                    }
                },
                Error(err) => return Error(err),
                Failure => return Success(results),
            }
        }
    }
}
