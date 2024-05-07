//! ## Reference
//!
//! Here's a quick reference table of the types of all the parser combinators.
//!
//! ```text
//! COMBINATOR           OUTPUT-TYPE    NOTES
//!
//! ~~ lexemes ~~
//! p_nothing()          ()
//! p_string()           ()
//! p_regex()            ()
//!
//! ~~ mapping ~~
//! P.constant(V)        V
//! P.fail(msg)          !
//! P.cut(Q)             (P, Q)
//! P.map_err(f)         f(P)
//!
//! P.substr(f)          f(&str)
//! P.map(f)             f(P)
//! P.span(f)            f(Span)
//! P.map_span(f)        f(Span, P)
//!
//! (P, Q)               (P, Q)
//! (P, Q, R)            (P, Q, R)
//! ...                  ...
//!
//! ~~ Error Handling ~~
//! P.resolve()          eliminate Result
//!
//! ~~ Sharing ~~
//! P.refn()             P
//!
//! ~~ repetition ~~
//! P.opt()              Option<P>
//! P.many0()            Vec<P>
//! P.many1()            Vec<P>
//! P.fold_many0(V, f)   V              f: Fn(V, P) -> V
//! P.fold_many1(Q, f)   P              f: Fn(P, Q) -> P
//! P.many_sep0(Q)       Vec<P>
//! P.many_sep1(Q)       Vec<P>
//!
//! ~~ other ~~
//! (P1, ..., Pn)
//!   Makes a parser of output type (P1, ..., Pn)
//!   Backtracks! Use P1.cut((P2, ..., Pn)) to not backtrack if P1 succeeds.
//! alt(name, (P1, ..., Pn))
//!   Try each parser in turn, using the first that succeeds.
//!   Requires that they all have the same output type.
//! alt_longest(name, (P1, ..., Pn))
//!   Try each parser in turn, using the longest successful match,
//!   with ties won by the earlier parser.
//!   Requires that they all have the same output type.
//! P.resolve()
//!   Converts Parser<Result<T, E>> into Parser<E> by shifting the error
//!   into the parse result.
//!
//! ~~ recursion ~~
//! For recursion, use the impl of Parser for functions:
//!     impl<T, F> Parser<T> for F
//!     where F: Fn(&mut Cursor, bool) -> Result<T, Option<ParseError>>,
//! ```

mod cursor;
mod parse_error;
mod pos;

pub use cursor::Cursor;
pub use parse_error::ParseError;
pub use pos::{Pos, Span};

use regex::{Error as RegexError, Regex};

pub fn parse<T>(filename: &str, source: &str, parser: impl Parser<T>) -> Result<T, ParseError> {
    let mut cursor = Cursor {
        filename: filename.to_owned(),
        source,
        pos: Pos::new(),
    };
    match parser.parse(&mut cursor, true) {
        Ok(succ) => {
            if cursor.is_at_end() {
                Ok(succ)
            } else {
                Err(cursor.error("expected end of file".to_owned()))
            }
        }
        Err(Some(err)) => Err(err),
        Err(None) => {
            Err(cursor
                .error("Invalid parser returned no error even though required=true".to_owned()))
        }
    }
}

/// Workaround for the fact that Rust doesn't yet have type equality constraints.
pub trait IsResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IsResult<T, E> for Result<T, E> {
    fn into_result(self) -> Result<T, E> {
        self
    }
}

/*==============*
 * Parser Trait *
 *==============*/

pub trait Parser<T> {
    #[doc(hidden)]
    fn parse(&self, cursor: &mut Cursor, required: bool) -> Result<T, Option<ParseError>>;

    /*=========*
     * Mapping *
     *=========*/

    fn constant(self, val: T) -> impl Parser<T>
    where
        Self: Sized,
        T: Clone,
    {
        move |cursor: &mut Cursor, required: bool| match self.parse(cursor, required) {
            Ok(_) => Ok(val.clone()),
            Err(err) => Err(err),
        }
    }

    fn fail(self, message: &str) -> impl Parser<T>
    where
        Self: Sized,
    {
        let message = message.to_owned();
        move |cursor: &mut Cursor, required: bool| {
            let start = cursor.pos;
            self.parse(cursor, required)?;
            Err(Some(cursor.error_from(message.clone(), start)))
        }
    }

    fn cut<T2>(self, other: impl Parser<T2>) -> impl Parser<(T, T2)>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let res_1 = self.parse(cursor, required)?;
            let res_2 = other.parse(cursor, true)?;
            Ok((res_1, res_2))
        }
    }

    fn map_err(self, func: impl Fn(String) -> String) -> impl Parser<T>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            self.parse(cursor, required)
                .map_err(|opt| opt.map(|err| err.map(&func)))
        }
    }

    fn map<T2>(self, func: impl Fn(T) -> T2) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| self.parse(cursor, required).map(&func)
    }

    fn resolve<T2, E>(self) -> impl Parser<T2>
    where
        T: IsResult<T2, E>,
        E: std::error::Error,
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let start = cursor.pos;
            let result = self.parse(cursor, required)?;
            match result.into_result() {
                Ok(succ) => Ok(succ),
                Err(err) if required => Err(Some(cursor.error_from(err.to_string(), start))),
                Err(_) => Err(None),
            }
        }
    }

    fn substr<T2>(self, func: impl Fn(&str) -> T2) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let start = cursor.pos;
            self.parse(cursor, required)?;
            Ok(func(cursor.substr_from(start)))
        }
    }

    fn span<T2>(self, func: impl Fn(Span) -> T2) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let start = cursor.pos;
            self.parse(cursor, required)?;
            Ok(func(cursor.span_from(start)))
        }
    }

    fn map_span<T2>(self, func: impl Fn(T, Span) -> T2) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let start = cursor.pos;
            let val = self.parse(cursor, required)?;
            Ok(func(val, cursor.span_from(start)))
        }
    }

    /*=========*
     * Sharing *
     *=========*/

    fn refn<'a>(&'a self) -> impl Parser<T> + Copy + 'a
    where
        Self: Sized,
    {
        struct PRefn<'a, P>(&'a P);

        impl<'a, P> Clone for PRefn<'a, P> {
            fn clone(&self) -> PRefn<'a, P> {
                PRefn(self.0)
            }
        }

        impl<'a, P> Copy for PRefn<'a, P> {}

        impl<'a, T, P> Parser<T> for PRefn<'a, P>
        where
            P: Parser<T>,
        {
            fn parse(&self, cursor: &mut Cursor, required: bool) -> Result<T, Option<ParseError>> {
                self.0.parse(cursor, required)
            }
        }

        PRefn(self)
    }

    /*============*
     * Repetition *
     *============*/

    fn opt(self) -> impl Parser<Option<T>>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, _required: bool| match self.parse(cursor, false) {
            Ok(succ) => Ok(Some(succ)),
            Err(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

/*=========*
 * Parsers *
 *=========*/

impl<T, F> Parser<T> for F
where
    F: Fn(&mut Cursor, bool) -> Result<T, Option<ParseError>>,
{
    fn parse(&self, cursor: &mut Cursor, required: bool) -> Result<T, Option<ParseError>> {
        self(cursor, required)
    }
}

/*=========*
 * Lexemes *
 *=========*/

pub fn nothing() -> impl Parser<()> {
    move |_cursor: &mut Cursor, _required: bool| Ok(())
}

pub fn string(expected: &str) -> impl Parser<()> {
    let delta_pos = Pos::delta(expected);
    let expected = expected.to_owned();
    move |cursor: &mut Cursor, required: bool| {
        if cursor.consume_str(&expected, delta_pos) {
            Ok(())
        } else if required {
            Err(Some(cursor.error(format!("expected {expected}"))))
        } else {
            Err(None)
        }
    }
}

pub fn regex(label: &str, regex_str: &str) -> Result<impl Parser<()>, RegexError> {
    let regex = new_regex(regex_str)?;
    let label = label.to_owned();
    Ok(move |cursor: &mut Cursor, required: bool| {
        if cursor.consume_regex(&regex) {
            Ok(())
        } else if required {
            Err(Some(cursor.error(format!("expected {label}"))))
        } else {
            Err(None)
        }
    })
}

/*========*
 * Choice *
 *========*/

pub fn alt<T>(label: &str, options: impl AltTuple<T>) -> impl Parser<T> {
    options.make_alt(label.to_owned())
}

pub fn alt_longest<T>(label: &str, options: impl AltTuple<T>) -> impl Parser<T> {
    options.make_alt_longest(label.to_owned())
}

pub trait AltTuple<T> {
    fn make_alt(self, label: String) -> impl Parser<T>;
    fn make_alt_longest(self, label: String) -> impl Parser<T>;
}

macro_rules! define_alt {
    ($struct:ident, $type:ident, $( ($idx:tt, $parser:ident) ),*) => {
        struct $struct<$( $parser ),*>($( $parser ),*);

        impl<$type, $( $parser ),*> AltTuple<$type> for $struct<$( $parser ),*>
        where $( $parser : Parser<$type> ),* {
            fn make_alt(self, label: String) -> impl Parser<T> {
                move |cursor: &mut Cursor, required: bool| {
                    let start = cursor.pos;
                    $(
                        match self.$idx.parse(cursor, false) {
                            Ok(succ) => return Ok(succ),
                            Err(Some(err)) => return Err(Some(err)),
                            Err(None) => cursor.pos = start,
                        }
                    )*
                    if required {
                        Err(Some(cursor.error(format!("expected {label}"))))
                    } else {
                        Err(None)
                    }
                }
            }

            fn make_alt_longest(self, label: String) -> impl Parser<T> {
                move |cursor: &mut Cursor, required: bool| {
                    let start = cursor.pos;
                    let mut best = None;
                    $(
                        match self.$idx.parse(cursor, false) {
                            Ok(succ) => {
                                let len = cursor.pos.offset - start.offset;
                                if let Some((_, best_len)) = &best {
                                    if len > *best_len {
                                        best = Some((succ, len));
                                    }
                                } else {
                                    best = Some((succ, len));
                                }
                                cursor.pos = start;
                            }
                            Err(Some(err)) => return Err(Some(err)),
                            Err(None) => cursor.pos = start,
                        }
                    )*
                    if let Some((succ, _)) = best {
                        Ok(succ)
                    } else if required {
                        Err(Some(cursor.error(format!("expected {label}"))))
                    } else {
                        Err(None)
                    }
                }
            }
        }
    }
}

define_alt!(AltTuple2, T, (0, P0), (1, P1));
define_alt!(AltTuple3, T, (0, P0), (1, P1), (2, P2));
define_alt!(AltTuple4, T, (0, P0), (1, P1), (2, P2), (3, P3));
define_alt!(AltTuple5, T, (0, P0), (1, P1), (2, P2), (3, P3), (4, P4));
define_alt!(
    AltTuple6,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5)
);
define_alt!(
    AltTuple7,
    T,
    (0, P0),
    (1, P1),
    (2, P2),
    (3, P3),
    (4, P4),
    (5, P5),
    (6, P6)
);
define_alt!(
    AltTuple8,
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

/*==========*
 * Sequence *
 *==========*/

macro_rules! define_seq {
    ($( ($var:ident, $idx:tt, $type:ident, $parser:ident) ),*) => {
        impl<$( $type, $parser ),*> Parser<($( $type ),*)> for ($( $parser ),*)
        where $( $parser : Parser<$type>),* {
            fn parse(
                &self,
                cursor: &mut Cursor,
                required: bool
            ) -> Result<($( $type ),*), Option<ParseError>> {
                $(
                    let $var = self.$idx.parse(cursor, required)?;
                )*
                Ok(($( $var ),*))
            }
        }
    };
}

define_seq!((v0, 0, T0, P0), (v1, 1, T1, P1));
define_seq!((v0, 0, T0, P0), (v1, 1, T1, P1), (v2, 2, T2, P2));
define_seq!(
    (v0, 0, T0, P0),
    (v1, 1, T1, P1),
    (v2, 2, T2, P2),
    (v3, 3, T3, P3)
);
define_seq!(
    (v0, 0, T0, P0),
    (v1, 1, T1, P1),
    (v2, 2, T2, P2),
    (v3, 3, T3, P3),
    (v4, 4, T4, P4)
);
define_seq!(
    (v0, 0, T0, P0),
    (v1, 1, T1, P1),
    (v2, 2, T2, P2),
    (v3, 3, T3, P3),
    (v4, 4, T4, P4),
    (v5, 5, T5, P5)
);
define_seq!(
    (v0, 0, T0, P0),
    (v1, 1, T1, P1),
    (v2, 2, T2, P2),
    (v3, 3, T3, P3),
    (v4, 4, T4, P4),
    (v5, 5, T5, P5),
    (v6, 6, T6, P6)
);
define_seq!(
    (v0, 0, T0, P0),
    (v1, 1, T1, P1),
    (v2, 2, T2, P2),
    (v3, 3, T3, P3),
    (v4, 4, T4, P4),
    (v5, 5, T5, P5),
    (v6, 6, T6, P6),
    (v7, 7, T7, P7)
);

/*======*
 * Misc *
 *======*/

fn new_regex(regex_str: &str) -> Result<Regex, RegexError> {
    match Regex::new(&format!("^({})", regex_str)) {
        Ok(regex) => Ok(regex),
        Err(err) => match Regex::new(regex_str) {
            // This error message is better because it doesn't have the ^({}) wrapper in it.
            Err(err) => Err(err),
            // Not sure why this wasn't an error too but there's still an issue.
            Ok(_) => Err(err),
        },
    }
}
