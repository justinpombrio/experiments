// TODO: temporary
#![allow(unused)]

// TODO: More combinators needed:
// - sep

// This design achieves all of the following:
//
// - The lexer isn't exposed (i.e. `Token` isn't in the interface).
// - The types of parsers is simple: `Parser<T>` instead of something less weildy
//   like `impl Parser<Output = T>` or even `Parser<impl Parse<Output = T>>`.
// - Its implementation of recursive parsers doesn't threaten to summon Cthulu.
// - It allows parsers to be cloned without having the illegal `Box<Trait + Clone>`.
// - Implementing a parser combinator isn't too onerous.
// - `InitialSet`s aren't needlessly cloned.
//
// Any change to the design is liable to break one of these properties, so if
// considering a change check this list first.

mod initial_set;
mod lexer;
mod seqs;
mod vec_map;

use crate::lexer::{LexemeIter, Lexer, LexerBuilder, Position, Token};
use crate::vec_map::VecMap;
use dyn_clone::{clone_box, clone_trait_object, DynClone};
use initial_set::{ChoiceTable, InitialSet};
use regex::Error as RegexError;
use regex::Regex;
use std::array;
use std::cell::OnceCell;
use std::error::Error;
use std::fmt;
use std::rc::{Rc, Weak};
use thiserror::Error;

pub use seqs::{seq3, seq4};

/*========================================*/
/*          Interface                     */
/*========================================*/

pub struct Parser<T> {
    initial_set: InitialSet,
    parse_fn: ParseFn<T>,
}

type ParseFn<T> = Box<dyn Parse<T>>;

trait Parse<T>: DynClone {
    fn parse(&self, stream: &mut LexemeIter) -> Result<T, ParseError>;
}

impl<T> Clone for Parser<T> {
    fn clone(&self) -> Parser<T> {
        Parser {
            initial_set: self.initial_set.clone(),
            parse_fn: clone_box(self.parse_fn.as_ref()),
        }
    }
}

impl<T> Parser<T> {
    fn new<P: Parse<T> + 'static>(initial_set: InitialSet, parse: P) -> Parser<T> {
        Parser {
            initial_set,
            parse_fn: Box::new(parse),
        }
    }
}

/*========================================*/
/*          Parse Errors                  */
/*========================================*/

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    CustomError(String),
    #[error("Parse error: expected {expected} but found '{found}'")]
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
/*          Grammar                       */
/*========================================*/

#[derive(Clone)]
pub struct Grammar(LexerBuilder);

impl Grammar {
    pub fn new(whitespace_regex: &str) -> Result<Grammar, GrammarError> {
        let lexer_builder =
            LexerBuilder::new(whitespace_regex).map_err(GrammarError::RegexError)?;
        Ok(Grammar(lexer_builder))
    }

    pub fn string(&mut self, string: &str) -> Result<Parser<()>, GrammarError> {
        let name = format!("'{}'", string);
        let token = self.0.string(string)?;
        Ok(Parser {
            initial_set: InitialSet::new_token(name.clone(), token),
            parse_fn: Box::new(TokenP { name, token }),
        })
    }

    pub fn regex(&mut self, name: &str, regex: &str) -> Result<Parser<()>, GrammarError> {
        let name = name.to_owned();
        let token = self.0.regex(regex)?;
        Ok(Parser {
            initial_set: InitialSet::new_token(name.clone(), token),
            parse_fn: Box::new(TokenP { name, token }),
        })
    }

    pub fn make_parse_fn<T: Clone>(
        &self,
        parser: Parser<T>,
    ) -> impl Fn(&str) -> Result<T, ParseError> + Clone {
        // TODO: ensure whole stream is consumed!
        let lexer = self.clone().0.finish();
        move |input: &str| {
            // By default, this closure captures `&parser.0`, which doesn't
            // implement `Clone`. Force it to capture `&parser` instead.
            let parser = &parser;

            let mut lexemes = lexer.lex(input);
            parser.parse_fn.parse(&mut lexemes)
        }
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
/*          Parser: Token                 */
/*========================================*/

#[derive(Clone)]
struct TokenP {
    name: String,
    token: Token,
}

impl Parse<()> for TokenP {
    fn parse(&self, stream: &mut LexemeIter) -> Result<(), ParseError> {
        if let Some(lexeme) = stream.peek() {
            if lexeme.token == self.token {
                stream.next();
                return Ok(());
            }
        }
        Err(ParseError::new(
            &self.name,
            stream.next().map(|lex| lex.lexeme),
        ))
    }
}

/*========================================*/
/*          Parser: Empty                 */
/*========================================*/

#[derive(Clone)]
struct EmptyP;

impl Parse<()> for EmptyP {
    fn parse(&self, stream: &mut LexemeIter) -> Result<(), ParseError> {
        Ok(())
    }
}

pub fn empty() -> Parser<()> {
    Parser {
        initial_set: InitialSet::new_empty("empty"),
        parse_fn: Box::new(EmptyP),
    }
}

/*========================================*/
/*          Parser: Map                   */
/*========================================*/

struct MapP<I: Clone, O: Clone, F: Fn(I) -> O + Clone> {
    parse_fn: ParseFn<I>,
    func: F,
}

impl<I: Clone, O: Clone, F: Fn(I) -> O + Clone> Clone for MapP<I, O, F> {
    fn clone(&self) -> Self {
        MapP {
            parse_fn: clone_box(self.parse_fn.as_ref()),
            func: self.func.clone(),
        }
    }
}

impl<I: Clone, O: Clone, F: Fn(I) -> O + Clone> Parse<O> for MapP<I, O, F> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<O, ParseError> {
        let result = self.parse_fn.parse(stream)?;
        Ok((self.func)(result))
    }
}

impl<T: Clone + 'static> Parser<T>
where
    Self: Clone,
{
    pub fn map<O: Clone + 'static>(self, func: impl Fn(T) -> O + Clone + 'static) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(MapP {
                parse_fn: self.parse_fn,
                func,
            }),
        }
    }

    pub fn value<O: Clone + 'static>(self, value: O) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(MapP {
                parse_fn: self.parse_fn,
                func: move |_| value.clone(),
            }),
        }
    }
}

/*========================================*/
/*          Parser: Span                  */
/*========================================*/

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span<'s> {
    pub substring: &'s str,
    pub start: Position,
    pub end: Position,
}

impl<'s> fmt::Display for Span<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

struct TrySpanP<I: Clone, O: Clone, F: Fn(Span, I) -> Result<O, String> + Clone> {
    parse_fn: ParseFn<I>,
    func: F,
}

impl<I: Clone, O: Clone, F: Fn(Span, I) -> Result<O, String> + Clone> Clone for TrySpanP<I, O, F> {
    fn clone(&self) -> Self {
        TrySpanP {
            parse_fn: clone_box(self.parse_fn.as_ref()),
            func: self.func.clone(),
        }
    }
}

impl<I: Clone, O: Clone, F: Fn(Span, I) -> Result<O, String> + Clone> Parse<O>
    for TrySpanP<I, O, F>
{
    fn parse(&self, stream: &mut LexemeIter) -> Result<O, ParseError> {
        let start = stream.pos();
        let source = stream.remaining_source();
        let result = self.parse_fn.parse(stream)?;
        let end = stream.pos();
        let len = end.pos - start.pos;
        let span = Span {
            substring: &source[0..len],
            start,
            end,
        };
        (self.func)(span, result).map_err(ParseError::CustomError)
    }
}

impl<T: Clone + 'static> Parser<T>
where
    Self: Clone,
{
    pub fn span<O: Clone + 'static>(self, func: impl Fn(Span) -> O + Clone + 'static) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(TrySpanP {
                parse_fn: self.parse_fn,
                func: move |span, _| Ok(func(span)),
            }),
        }
    }

    pub fn map_span<O: Clone + 'static>(
        self,
        func: impl Fn(Span, T) -> O + Clone + 'static,
    ) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(TrySpanP {
                parse_fn: self.parse_fn,
                func: move |span, val| Ok(func(span, val)),
            }),
        }
    }

    pub fn try_span<O: Clone + 'static>(
        self,
        func: impl Fn(Span) -> Result<O, String> + Clone + 'static,
    ) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(TrySpanP {
                parse_fn: self.parse_fn,
                func: move |span, _| func(span),
            }),
        }
    }

    pub fn try_map_span<O: Clone + 'static>(
        self,
        func: impl Fn(Span, T) -> Result<O, String> + Clone + 'static,
    ) -> Parser<O> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(TrySpanP {
                parse_fn: self.parse_fn,
                func,
            }),
        }
    }
}

/*========================================*/
/*          Parser: Seq2                  */
/*========================================*/

struct Seq2P<T0, T1>(ParseFn<T0>, ParseFn<T1>);

impl<T0: Clone, T1: Clone> Clone for Seq2P<T0, T1> {
    fn clone(&self) -> Self {
        Seq2P(clone_box(self.0.as_ref()), clone_box(self.1.as_ref()))
    }
}

impl<T0: Clone, T1: Clone> Parse<(T0, T1)> for Seq2P<T0, T1> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<(T0, T1), ParseError> {
        let result_0 = self.0.parse(stream)?;
        let result_1 = self.1.parse(stream)?;
        Ok((result_0, result_1))
    }
}

impl<T0: Clone + 'static> Parser<T0> {
    pub fn and<T1: Clone + 'static>(
        self,
        other: Parser<T1>,
    ) -> Result<Parser<(T0, T1)>, GrammarError> {
        seq2(self, other)
    }

    pub fn and_ignore<T1: Clone + 'static>(
        self,
        other: Parser<T1>,
    ) -> Result<Parser<T0>, GrammarError> {
        Ok(seq2(self, other)?.map(|(v0, _)| v0))
    }
}

pub fn seq2<T0: Clone + 'static, T1: Clone + 'static>(
    parser_0: Parser<T0>,
    parser_1: Parser<T1>,
) -> Result<Parser<(T0, T1)>, GrammarError> {
    let mut initial_set = parser_0.initial_set;
    initial_set.seq(parser_1.initial_set)?;
    Ok(Parser {
        initial_set,
        parse_fn: Box::new(Seq2P(parser_0.parse_fn, parser_1.parse_fn)),
    })
}

/*========================================*/
/*          Parser: Choice                */
/*========================================*/

struct ChoiceP<T, const N: usize> {
    name: String,
    choice_table: ChoiceTable,
    parse_fns: [ParseFn<T>; N],
}

impl<T: Clone, const N: usize> Clone for ChoiceP<T, N> {
    fn clone(&self) -> Self {
        ChoiceP {
            name: self.name.clone(),
            choice_table: self.choice_table.clone(),
            parse_fns: array::from_fn(|i| clone_box(self.parse_fns[i].as_ref())),
        }
    }
}

impl<T: Clone, const N: usize> Parse<T> for ChoiceP<T, N> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<T, ParseError> {
        let lexeme = stream.peek();
        match self.choice_table.lookup(lexeme.map(|lex| lex.token)) {
            None => Err(ParseError::new(&self.name, lexeme.map(|lex| lex.lexeme))),
            Some(i) => self.parse_fns[i].parse(stream),
        }
    }
}

pub fn choice<T: Clone + 'static, const N: usize>(
    name: &str,
    parsers: [Parser<T>; N],
) -> Result<Parser<T>, GrammarError> {
    // Would be nice to avoid this copy, but I don't think you can do that
    // in Rust right now?
    let initial_sets = parsers
        .iter()
        .map(|p| p.initial_set.clone())
        .collect::<Vec<_>>();
    let parse_fns = parsers.map(|p| p.parse_fn);
    let (choice_table, initial_set) = ChoiceTable::new(name, initial_sets)?;
    Ok(Parser {
        initial_set,
        parse_fn: Box::new(ChoiceP {
            name: name.to_owned(),
            choice_table,
            parse_fns,
        }),
    })
}

/*========================================*/
/*          Parser: Optional              */
/*========================================*/

impl<T: Clone + 'static> Parser<T> {
    pub fn opt(self) -> Result<Parser<Option<T>>, GrammarError> {
        let name = self.initial_set.name().to_owned();
        choice(&name, [empty().map(|()| None), self.map(Some)])
    }
}

/*========================================*/
/*          Parser: Many                  */
/*========================================*/

struct ManyP<T> {
    name: String,
    initial_tokens: VecMap<()>,
    parse_fn: ParseFn<T>,
}

impl<T: Clone> Clone for ManyP<T> {
    fn clone(&self) -> Self {
        ManyP {
            name: self.name.clone(),
            initial_tokens: self.initial_tokens.clone(),
            parse_fn: clone_box(self.parse_fn.as_ref()),
        }
    }
}

impl<T: Clone> Parse<Vec<T>> for ManyP<T> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<Vec<T>, ParseError> {
        let mut result = Vec::new();
        while let Some(lexeme) = stream.peek() {
            if self.initial_tokens.get(lexeme.token).is_some() {
                result.push(self.parse_fn.parse(stream)?);
            } else {
                break;
            }
        }
        Ok(result)
    }
}

impl<T: Clone + 'static> Parser<T> {
    pub fn many(self) -> Result<Parser<Vec<T>>, GrammarError> {
        let name = self.initial_set.name().to_owned();
        let initial_tokens = self.initial_set.accepted_tokens();
        let mut initial_set = self.initial_set;
        initial_set.union("many", InitialSet::new_empty("empty"))?;
        Ok(Parser {
            initial_set,
            parse_fn: Box::new(ManyP {
                name,
                initial_tokens,
                parse_fn: self.parse_fn,
            }),
        })
    }

    pub fn sep<U: Clone + 'static>(self, sep: Parser<U>) -> Result<Parser<Vec<T>>, GrammarError> {
        let sep_elem = sep.and(self.clone())?.map(|(_, v)| v);
        Ok(self
            .clone()
            .and(sep_elem.many()?)?
            .map(|(last, mut vec)| {
                vec.insert(0, last);
                vec
            })
            .opt()?
            .map(|opt| opt.unwrap_or_else(|| Vec::new())))
    }
}

/*========================================*/
/*          Parser: Complete              */
/*========================================*/

struct CompleteP<T> {
    parse_fn: ParseFn<T>,
}

impl<T: Clone> Clone for CompleteP<T> {
    fn clone(&self) -> Self {
        CompleteP {
            parse_fn: clone_box(self.parse_fn.as_ref()),
        }
    }
}

impl<T: Clone> Parse<T> for CompleteP<T> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<T, ParseError> {
        let result = self.parse_fn.parse(stream)?;
        if let Some(lexeme) = stream.peek() {
            Err(ParseError::new("end of file", Some(lexeme.lexeme)))
        } else {
            Ok(result)
        }
    }
}

impl<T: Clone + 'static> Parser<T> {
    pub fn complete(self) -> Parser<T> {
        Parser {
            initial_set: self.initial_set,
            parse_fn: Box::new(CompleteP {
                parse_fn: self.parse_fn,
            }),
        }
    }
}

/*========================================*/
/*          Parser: Recur                 */
/*========================================*/

#[derive(Clone)]
pub struct Recur<T>(Rc<OnceCell<Parser<T>>>);

impl<T: Clone> Parse<T> for Recur<T> {
    fn parse(&self, stream: &mut LexemeIter) -> Result<T, ParseError> {
        self.0.get().unwrap().parse_fn.parse(stream)
    }
}

pub fn recur<T: Clone + 'static>(
    make_parser: impl FnOnce(Parser<T>) -> Result<Parser<T>, GrammarError>,
) -> Result<Parser<T>, GrammarError> {
    // TODO: Make sure this ever gets dropped. Needs weak?
    let cell = Rc::new(OnceCell::new());
    let recur = Recur(cell.clone());
    let inner_parser = Parser {
        initial_set: InitialSet::new_void("recur"),
        parse_fn: Box::new(recur),
    };
    let outer_parser = make_parser(inner_parser)?;
    cell.set(outer_parser.clone());
    Ok(outer_parser)
}
