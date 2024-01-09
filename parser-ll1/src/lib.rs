// TODO:
// [x] Make something nicer than seq_n and choice_n for users
// [x] Have recur's interface use mutation, and panic on drop
// [x] Have recur validate, but only to depth 2, using an atomic u8
// [x] ParseError: fancy underlines
// [x] GrammarError: fix message on choice
// [x] Test errors: give line number, better error message
// [ ] Review&test error messages
// [x] Review combinator names
// [ ] Add iterator combinator for streaming parsing?
// [ ] Add context() combinator?
// [x] Change Parser<Output = T> to Parser<T>
// [x] Try having parsers lex directly instead of having a separate lexer;
//     see if that dramatically improves the speed. Yes: by 20%.
// [x] Make things run on stable Rust, it's a feature. Don't need `trait =`.
// [x] Docs
// [ ] Testing
// [ ] Review docs, add example

// NOTE: Current time to parse dummy.json: 6.5 ms

// This design achieves all of the following:
//
// - The lexer isn't exposed (i.e. `Token` isn't in the interface).
// - The types of parsers is nice `impl Parser<T>`.
// - The implementation of recursive parsers doesn't threaten to summon Cthulu.
// - Parsers can be cloned without having the illegal `Box<Trait + Clone>`.
// - Implementing a parser combinator isn't too onerous.
// - `InitialSet`s aren't needlessly cloned (except if you call `compile_parser`
//   many times, but whatever).
// - No unnecessary boxing.
//
// Any change to the design is liable to break one of these properties, so if
// considering a change check this list first.
//
// It's tempting to remove the lexer. Doing so yields a ~20% speedup, but it
// would make the parse error messages worse. Not worth it!

//! # parser_ll1
//!
//! **Guaranteed linear time parsing with typed parser combinators.**
//!
//! ```
//! use parser_ll1::{Grammar, Parser, CompiledParser};
//! use std::str::FromStr;
//!
//! let mut g = Grammar::with_whitespace("[ \t\n]+").unwrap();
//! let number = g.regex("number", "[0-9]+").unwrap()
//!     .try_span(|s| i32::from_str(s.substr));
//! let numbers = number.many_sep1(g.string("+").unwrap())
//!     .map(|nums| nums.into_iter().sum());
//! let parser = g.compile_parser(numbers).unwrap();
//!
//! assert_eq!(parser.parse("test case", "1 + 2 + 3"), Ok(6));
//! assert_eq!(format!("{}", parser.parse("test case", "1 + + 2").unwrap_err()),
//! "Parse error: expected number but found '+'.
//! At 'test case' line 1:
//!
//! 1 + + 2
//!     ^");
//! ```
//!
//! ## Features
//!
//! - Guaranteed linear time parsing, due to `parse_ll1` checking that
//!   your grammar is LL1. You won't find guaranteed linear time parsing in
//!   any other (complete) Rust parser library. `nom` and `parsell` can take
//!   exponential time to parse if they're given a poorly structured grammar.
//! - Typed parser combinators, so that you build your parser in Rust code,
//!   and it produces a result directly. You don't have to write separate
//!   code to walk a parse tree like in [`pest`](https://pest.rs/).
//! - Good error messages, with no effort required from you. This is due
//!   to the fact that `parer_ll1` enforces that grammars are LL1 (so it
//!   always knows exactly what's being parsed), and that under the hood
//!   it lexes before it parses (so it knows what to point at if the next
//!   token is unexpected).
//! - Easier to use than `nom` or `pest`.
//! - Runs on stable Rust.
//!
//! ## Non-features
//!
//! - Grammars that aren't LL1! (In the future I may add backtracking
//!   versions of some of the combinators, which would allow parsing
//!   non-LL1 grammars in exchange for losing the nice guarantees
//!   the LL1 property gives you.)
//! - Byte-level parsing. Use [`nom`](https://docs.rs/nom/latest/nom/)
//!   instead.
//! - Streaming parsing. Use [`nom`](https://docs.rs/nom/latest/nom/)
//!   or [`parsell`](https://docs.rs/parsell/latest/parsell/) instead.
//! - Extraordinary speed. Use [`nom`](https://docs.rs/nom/latest/nom/)
//!   instead. A preliminary benchmark puts `parse_ll1` at ~half the speed
//!   of `nom`, which to be clear is still very fast.
//! - There's no separate grammar file, which some people like because
//!   it's so declarative. Use [`pest`](https://pest.rs/) instead.

mod initial_set;
mod lexer;
mod parse_error;
mod parser_recur;
mod vec_map;

use dyn_clone::{clone_box, DynClone};
use lexer::{LexemeIter, Lexer, LexerBuilder, Token, TOKEN_EOF};
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

use initial_set::InitialSet;
pub use lexer::Position;
pub use parse_error::ParseError;
pub use parser_recur::Recursive;

#[doc(hidden)]
pub enum ParseResult<T> {
    Success(T),
    Failure,
    Error(ParseErrorCause),
}

/// A parser that outputs type `T` on a successful parse.
///
/// You cannot use this parser directly; you must call [`Grammar::compile_parser`] first.
pub trait Parser<T>: DynClone {
    /// A descriptive name for this parser. Used in error messages.
    fn name(&self) -> String;
    #[doc(hidden)]
    fn validate(&self) -> Result<InitialSet, GrammarError>;
    #[doc(hidden)]
    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T>;

    // NOTE: used to have a few more combinators, though I removed them
    // because they lacked names and thus would produce worse error messages:
    //
    // - and
    // - preceded
    // - terminated
    // - complete (implicitly inserted by `compile_parser()`)

    /// Ignore this parser's output, replacing it with `value`.
    fn constant<T2: Clone>(self, value: T2) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.map(move |_| value.clone())
    }

    /// Transform this parser's output value with `func`.
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

    /// Transform this parser's output value with `func`, producing a parse
    /// error if `func` returns an `Err`.
    fn try_map<T2, E: error::Error>(
        self,
        func: impl Fn(T) -> Result<T2, E> + Clone,
    ) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        TryMapP {
            parser: self,
            func,
            phantom: PhantomData,
        }
    }

    /// Ignore this parser's output, replacing it with `func(Span)` instead.
    ///
    /// The `Span` gives the region of the input text which was matched, including
    /// the substring.
    fn span<T2>(self, func: impl Fn(Span) -> T2 + Clone) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.map_span(move |span, _| func(span))
    }

    /// Combine this parser's output (of type `T`) together with the matched `Span`,
    /// to produce an output of type `T2`.
    ///
    /// The `Span` gives the region of the input text which was matched, including
    /// the substring.
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

    /// Ignore this parser's output, replacing it with `func(Span)` instead.
    /// Produce a parse error if `func` returns an `Err`.
    ///
    /// The `Span` gives the region of the input text which was matched, including
    /// the substring.
    fn try_span<T2, E: error::Error>(
        self,
        func: impl Fn(Span) -> Result<T2, E> + Clone,
    ) -> impl Parser<T2> + Clone
    where
        Self: Clone,
    {
        self.try_map_span(move |span, _| func(span))
    }

    /// Combine this parser's output (of type `T`) together with the matched `Span`,
    /// to produce an output of type `T2`. Produce a parse error if `func` returns an `Err`.
    ///
    /// The `Span` gives the region of the input text which was matched, including
    /// the substring.
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

    /// Either parse `self`, or parse nothing.
    fn opt(self) -> impl Parser<Option<T>> + Clone
    where
        Self: Clone,
    {
        OptP(self, PhantomData)
    }

    /// Parse `self` zero or more times.
    fn many0(self) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        ManyP(self, PhantomData)
    }

    /// Parse `self`, followed by zero or more occurrences of `parser`.
    /// Combine the outputs using `fold`.
    fn fold_many1<T2>(
        self,
        parser: impl Parser<T2> + Clone,
        fold: impl Fn(T, T2) -> T + Clone,
    ) -> impl Parser<T> + Clone
    where
        Self: Clone,
    {
        Fold1P {
            first_parser: self,
            many_parser: parser,
            fold,
            phantom: PhantomData,
        }
    }

    /// Parse zero or more occurrences of `self`.
    /// Combine the outputs using `fold`.
    fn fold_many0<V: Clone>(
        self,
        initial_value: V,
        fold: impl Fn(V, T) -> V + Clone,
    ) -> impl Parser<V> + Clone
    where
        Self: Clone,
    {
        Fold0P {
            parser: self,
            initial_value,
            fold,
            phantom: PhantomData,
        }
    }

    /// Parse `self` one or more times.
    fn many1(self) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        // TODO: this could be more efficient.
        let name = format!("{}.many1()", self.name());
        tuple(&name, (self.clone(), self.many0())).map(|(val, mut vec)| {
            vec.insert(0, val);
            vec
        })
    }

    /// Parse `self` zero or more times, separated by `sep`s.
    ///
    /// Collects the `self` outputs into a vector, and ignores the `sep` outputs.
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

    /// Parse `self` one or more times, separated by `sep`s.
    ///
    /// Collects the `self` outputs into a vector, and ignores the `sep` outputs.
    fn many_sep1<T2>(self, sep: impl Parser<T2> + Clone) -> impl Parser<Vec<T>> + Clone
    where
        Self: Clone,
    {
        // TODO: this could be more efficient.
        let name = format!("{}.many_sep1()", sep.name());
        let sep_elem = tuple(&name, (sep, self.clone())).map(|(_, v)| v);
        tuple(&name, (self.clone(), sep_elem.many0())).map(|(last, mut vec)| {
            vec.insert(0, last);
            vec
        })
    }

    /// Parse `self` followed by `next`, producing a tuple of their outputs.
    fn and<T2>(self, next: impl Parser<T2> + Clone) -> impl Parser<(T, T2)> + Clone
    where
        Self: Clone,
    {
        tuple("'and'", (self, next))
    }

    /// Parse `prev` followed by `self`, keeping only the output of `self`.
    fn preceded<T2>(self, prev: impl Parser<T2> + Clone) -> impl Parser<T> + Clone
    where
        Self: Clone,
    {
        tuple("'and'", (prev, self)).map(|(_, v)| v)
    }

    /// Parse `self` followed by `next`, keeping only the output of `self`.
    fn terminated<T2>(self, next: impl Parser<T2> + Clone) -> impl Parser<T> + Clone
    where
        Self: Clone,
    {
        tuple("'and'", (self, next)).map(|(v, _)| v)
    }
}

impl<T> Clone for Box<dyn Parser<T>> {
    fn clone(&self) -> Self {
        clone_box(self.as_ref())
    }
}

impl<T> Parser<T> for Box<dyn Parser<T>> {
    fn name(&self) -> String {
        self.as_ref().name()
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

/// Start here! Used to create parsers for tokens, from which all other parsers are
/// built, and to "compile" a finished parser to get a parsing function.
#[derive(Debug, Clone)]
pub struct Grammar(LexerBuilder);

/// White space as defined by the Pattern_White_Space Unicode property.
const UNICODE_WHITESPACE_REGEX: &str =
    "[\\u0009\\u000A\\u000B\\u000C\\u000D\\u0020\\u0085\\u200E\\u200F\\u2028\\u2029]*";

/// A compiled parsing function, ready to use.
pub trait CompiledParser<T> {
    /// Parse the `input` text. `filename` is used only for error messages.
    fn parse(&self, filename: &str, input: &str) -> Result<T, ParseError>;
}

impl Grammar {
    /// Construct a new grammar that uses the `Pattern_White_Space` Unicode property
    /// for whitespace.
    pub fn new() -> Grammar {
        let lexer_builder = LexerBuilder::new(UNICODE_WHITESPACE_REGEX).unwrap();
        Grammar(lexer_builder)
    }

    /// Construct a new grammar with a custom regex for matching whitespace.
    pub fn with_whitespace(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar(lexer_builder))
    }

    /// Create a parser that matches a string exactly.
    ///
    /// If the input could parse as _either_ a [`Grammar::string`] or a [`Grammar::regex`],
    /// then (i) the longest match wins, and (ii) `string`s win ties.
    pub fn string(&mut self, string: &str) -> Result<impl Parser<()> + Clone, GrammarError> {
        let name = format!("'{}'", string);
        let token = self.0.string(string)?;
        Ok(TokenP { name, token })
    }

    /// Create a parser that matches a regex. The regex syntax is that from the
    /// [regex](https://docs.rs/regex/latest/regex/) crate.
    ///
    /// If the input could parse as _either_ a [`Grammar::string`] or a [`Grammar::regex`],
    /// then (i) the longest match wins, and (ii) `string`s win ties.
    pub fn regex(
        &mut self,
        name: &str,
        regex: &str,
    ) -> Result<impl Parser<()> + Clone, GrammarError> {
        let token = self.0.regex(name, regex)?;
        let name = name.to_owned();
        Ok(TokenP { name, token })
    }

    /// Validate that a parser is LL1, and if so produce a parsing function.
    ///
    /// Call this once but invoke the function it returns every time you parse.
    pub fn compile_parser<T2, P: Parser<T2> + Clone>(
        &self,
        parser: P,
    ) -> Result<impl CompiledParser<T2>, GrammarError> {
        use ParseResult::{Error, Failure, Success};

        struct CompiledParserImpl<T, P: Parser<T> + Clone> {
            lexer: Lexer,
            parser: P,
            phantom: PhantomData<T>,
        }

        impl<T, P: Parser<T> + Clone> CompiledParser<T> for CompiledParserImpl<T, P> {
            fn parse(&self, filename: &str, input: &str) -> Result<T, ParseError> {
                let mut lexeme_iter = self.lexer.lex(input);
                match self.parser.parse(&mut lexeme_iter) {
                    Success(succ) => Ok(succ),
                    // TODO: fixme
                    Failure => panic!("Bug in parser wrapper"),
                    Error(err) => Err(err.build_error(filename, input)),
                }
            }
        }

        let lexer = self.clone().0.finish();
        // ensure the whole stream is consuemd
        let parser = tuple(&parser.name(), (parser, eof())).map(|(v, _)| v);
        parser.validate()?;

        Ok(CompiledParserImpl {
            lexer,
            parser,
            phantom: PhantomData,
        })
    }
}

/// An issue with the grammar defined by a parser.
#[derive(Error, Debug)]
pub enum GrammarError {
    /// Invalid regex.
    #[error("{0}")]
    RegexError(#[from] RegexError),
    /// The defined grammar is not LL1: two alternatives accept empty.
    #[error("Ambiguous grammar: {name} could be either empty {case_1} or empty {case_2}.")]
    AmbiguityOnEmpty {
        name: String,
        case_1: String,
        case_2: String,
    },
    /// The defined grammar is not LL1: two alternatives accept the same start token.
    #[error("Ambiguous grammar: when parsing {name}, token {token} could start either {case_1} or {case_2}.")]
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
    fn name(&self) -> String {
        "nothing".to_owned()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::empty())
    }

    fn parse(&self, _stream: &mut LexemeIter) -> ParseResult<()> {
        ParseResult::Success(())
    }
}

/// The most boring parser, which parses nothing and outputs `()`.
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
    fn name(&self) -> String {
        self.name.clone()
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        Ok(InitialSet::token(self.name.clone(), self.token))
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<()> {
        #[cfg(feature = "flamegraphs")]
        span!("Token");

        if stream.peek().token == self.token {
            stream.next();
            return ParseResult::Success(());
        }
        ParseResult::Failure
    }
}

fn eof() -> impl Parser<()> + Clone {
    TokenP {
        name: "end of file".to_owned(),
        token: TOKEN_EOF,
    }
}

/*========================================*/
/*          Parser: Try Map               */
/*========================================*/

struct TryMapP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(T0) -> Result<T1, E1> + Clone,
{
    parser: P0,
    func: F,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0, T1, E1, F> Clone for TryMapP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(T0) -> Result<T1, E1> + Clone,
{
    fn clone(&self) -> TryMapP<T0, P0, T1, E1, F> {
        TryMapP {
            parser: self.parser.clone(),
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0, T1, E1, F> Parser<T1> for TryMapP<T0, P0, T1, E1, F>
where
    P0: Parser<T0> + Clone,
    E1: error::Error,
    F: Fn(T0) -> Result<T1, E1> + Clone,
{
    fn name(&self) -> String {
        self.parser.name()
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
                    ParseResult::Error(ParseErrorCause {
                        message: err.to_string(),
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
    fn name(&self) -> String {
        self.parser.name()
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
/*          Parser: Try Span              */
/*========================================*/

/// A region of the input text, provided by method [`Parser::span`] and friends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'s> {
    /// The input text from `start` to `end`.
    pub substr: &'s str,
    /// The start of the span, just before its first character.
    pub start: Position,
    /// The end of the span, just after its last character.
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
    fn name(&self) -> String {
        self.parser.name()
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
            Err(err) => ParseResult::Error(ParseErrorCause {
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
    fn name(&self) -> String {
        self.parser.name()
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

/// Parse a sequence of things in order, collecting their outputs in a tuple.
///
/// - `name` is used in error messages to refer to this parser.
/// - `tuple` is a tuple of parsers all with the same output type.
pub fn tuple<T>(name: &str, tuple: impl SeqTuple<T>) -> impl Parser<T> + Clone {
    tuple.make_seq(name.to_owned())
}

/// A tuple of parsers for [`tuple()`] to try. Each tuple element must be a parser.
/// They may have different output types. Can have length up to 10.
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
            fn name(&self) -> String {
                self.name.clone()
            }

            fn validate(&self) -> Result<InitialSet, GrammarError> {
                InitialSet::sequence(
                    self.name(),
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
                            let lexeme = stream.peek();
                            return Error(ParseErrorCause {
                                message: format!("expected {} but found {}", self.parsers.$idx.name(), stream.token_name(lexeme.token).to_owned()),
                                span: (lexeme.start, lexeme.end),
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

/// Parse exactly one of the options.
///
/// - `name` is used in error messages to refer to this `choice`.
/// - `tuple` is a tuple of parsers all with the same output type.
pub fn choice<T>(name: &str, tuple: impl ChoiceTuple<T>) -> impl Parser<T> + Clone {
    tuple.make_choice(name.to_owned())
}

/// A tuple of parsers for [`choice`] to try. Each tuple element must be a parser,
/// and they must all have the same output type. Can have length up to 10.
pub trait ChoiceTuple<T> {
    #[doc(hidden)]
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
            fn name(&self) -> String {
                self.name.clone()
            }

            fn validate(&self) -> Result<InitialSet, GrammarError> {
                InitialSet::choice(
                    self.name(),
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
    fn name(&self) -> String {
        format!("{}.opt()", self.0.name())
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // If `self.0` accepts empty then this union will produce an error.
        // Otherwise the initial set is simply `self.0`s initial set
        // together with empty.
        InitialSet::choice(self.name(), vec![InitialSet::empty(), self.0.validate()?])
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
    fn name(&self) -> String {
        format!("{}.many0()", self.0.name())
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        // If `self.0` accepts empty then this union will produce an error.
        // Otherwise the initial set is simply `self.0`s initial set
        // together with empty.
        InitialSet::choice(self.name(), vec![InitialSet::empty(), self.0.validate()?])
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
/*          Parser: Fold                  */
/*========================================*/

struct Fold1P<T0, P0, T1, P1, F>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    F: Fn(T0, T1) -> T0 + Clone,
{
    first_parser: P0,
    many_parser: P1,
    fold: F,
    phantom: PhantomData<(T0, T1)>,
}

impl<T0, P0, T1, P1, F> Clone for Fold1P<T0, P0, T1, P1, F>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    F: Fn(T0, T1) -> T0 + Clone,
{
    fn clone(&self) -> Self {
        Fold1P {
            first_parser: self.first_parser.clone(),
            many_parser: self.many_parser.clone(),
            fold: self.fold.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T0, P0, T1, P1, F> Parser<T0> for Fold1P<T0, P0, T1, P1, F>
where
    P0: Parser<T0> + Clone,
    P1: Parser<T1> + Clone,
    F: Fn(T0, T1) -> T0 + Clone,
{
    fn name(&self) -> String {
        format!(
            "{}.fold_many1({})",
            self.first_parser.name(),
            self.many_parser.name()
        )
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        let init_first = self.first_parser.validate()?;
        let init_many = self.many_parser.validate()?;
        InitialSet::sequence(
            self.name(),
            vec![
                init_first,
                InitialSet::choice(self.name(), vec![InitialSet::empty(), init_many])?,
            ],
        )
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<T0> {
        use ParseResult::{Error, Failure, Success};

        #[cfg(feature = "flamegraphs")]
        span!("Fold1");

        let mut result = match self.first_parser.parse(stream) {
            Success(succ) => succ,
            Error(err) => return Error(err),
            Failure => return Failure,
        };
        loop {
            match self.many_parser.parse(stream) {
                Success(succ) => result = (self.fold)(result, succ),
                Error(err) => return Error(err),
                Failure => return Success(result),
            }
        }
    }
}

struct Fold0P<T, P: Parser<T> + Clone, V: Clone, F: Fn(V, T) -> V + Clone> {
    parser: P,
    initial_value: V,
    fold: F,
    phantom: PhantomData<T>,
}

impl<T, P: Parser<T> + Clone, V: Clone, F: Fn(V, T) -> V + Clone> Clone for Fold0P<T, P, V, F> {
    fn clone(&self) -> Self {
        Fold0P {
            parser: self.parser.clone(),
            initial_value: self.initial_value.clone(),
            fold: self.fold.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T, P: Parser<T> + Clone, V: Clone, F: Fn(V, T) -> V + Clone> Parser<V> for Fold0P<T, P, V, F> {
    fn name(&self) -> String {
        format!("{}.fold_many0()", self.parser.name(),)
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        InitialSet::choice(
            self.name(),
            vec![InitialSet::empty(), self.parser.validate()?],
        )
    }

    fn parse(&self, stream: &mut LexemeIter) -> ParseResult<V> {
        use ParseResult::{Error, Failure, Success};

        #[cfg(feature = "flamegraphs")]
        span!("Fold0");

        let mut result = self.initial_value.clone();
        loop {
            match self.parser.parse(stream) {
                Success(succ) => result = (self.fold)(result, succ),
                Error(err) => return Error(err),
                Failure => return Success(result),
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
    fn name(&self) -> String {
        format!("{}.many_sep0({})", self.elem.name(), self.sep.name())
    }

    fn validate(&self) -> Result<InitialSet, GrammarError> {
        let elem_init = self.elem.validate()?;
        let sep_init = self.sep.validate()?;

        // SepBy(E, S) = (.|E(SE)*) ~= (.|E(.|SE))
        let name = self.name();
        let sep_elem = InitialSet::sequence(name.clone(), vec![sep_init, elem_init.clone()])?;
        let tail = InitialSet::choice(name.clone(), vec![InitialSet::empty(), sep_elem])?;
        let nonempty = InitialSet::sequence(name.clone(), vec![elem_init, tail])?;
        InitialSet::choice(name, vec![InitialSet::empty(), nonempty])
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
                        let lexeme = stream.peek();
                        return Error(ParseErrorCause {
                            message: format!(
                                "expected {} but found {}",
                                self.sep.name(),
                                stream.token_name(lexeme.token).to_owned()
                            ),
                            span: (lexeme.start, lexeme.end),
                        });
                    }
                },
                Error(err) => return Error(err),
                Failure => return Success(results),
            }
        }
    }
}
