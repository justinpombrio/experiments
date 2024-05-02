#![allow(dead_code)]

use regex::{Error as RegexError, Regex};
use std::ops::Add;

pub struct ParseError {
    message: String,
    pos: Pos,
}

/*=====*
 * Pos *
 *=====*/

pub type Offset = usize;
pub type Line = u32;
pub type Col = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    offset: Offset,
    line: Line,
    col: Col,
}

impl Pos {
    fn delta(s: &str) -> Pos {
        let mut line = 0;
        let mut col = 0;
        for ch in s.chars() {
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Pos {
            offset: s.len(),
            line,
            col,
        }
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self: Pos, other: Pos) -> Pos {
        Pos {
            offset: self.offset + other.offset,
            line: self.line + other.line,
            col: if other.line == 0 {
                other.col
            } else {
                self.col + other.col
            },
        }
    }
}

/*========*
 * Cursor *
 *========*/

pub struct Cursor<'a> {
    source: &'a str,
    pos: Pos,
}

impl<'a> Cursor<'a> {
    fn str(&self) -> &str {
        &self.source[self.pos.offset..]
    }

    #[must_use]
    fn consume_str(&mut self, prefix: &str, delta: Pos) -> bool {
        if self.str().starts_with(prefix) {
            self.pos = self.pos + delta;
            true
        } else {
            false
        }
    }

    #[must_use]
    fn consume_regex(&mut self, regex: &Regex) -> bool {
        if let Some(re_match) = regex.find(self.str()) {
            let delta = Pos::delta(re_match.as_str());
            self.pos = self.pos + delta;
            true
        } else {
            false
        }
    }
}

/*==============*
 * Parser Trait *
 *==============*/

pub trait Parser<T> {
    #[doc(hidden)]
    fn parse(&self, cursor: &mut Cursor, required: bool) -> Result<T, Option<ParseError>>;

    /*==============*
     * Constructors *
     *==============*/

    fn string(expected: &str) -> impl Parser<()> {
        let delta_pos = Pos::delta(expected);
        let expected = expected.to_owned();
        move |cursor: &mut Cursor, required: bool| {
            if cursor.consume_str(&expected, delta_pos) {
                Ok(())
            } else if required {
                Err(Some(ParseError {
                    message: format!("expected {expected}"),
                    pos: cursor.pos,
                }))
            } else {
                Err(None)
            }
        }
    }

    fn regex(label: &str, regex_str: &str) -> Result<impl Parser<()>, RegexError> {
        let regex = new_regex(regex_str)?;
        let label = label.to_owned();
        Ok(move |cursor: &mut Cursor, required: bool| {
            if cursor.consume_regex(&regex) {
                Ok(())
            } else if required {
                Err(Some(ParseError {
                    message: format!("expected {label}"),
                    pos: cursor.pos,
                }))
            } else {
                Err(None)
            }
        })
    }

    /*=========*
     * Methods *
     *=========*/

    fn map<T2>(self, func: impl Fn(T) -> T2) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| self.parse(cursor, required).map(&func)
    }

    fn flat_map<P, T2>(self, func: impl Fn(T) -> P) -> impl Parser<T2>
    where
        P: Parser<T2>,
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let out = self.parse(cursor, required)?;
            let parser = func(out);
            parser.parse(cursor, required)
        }
    }

    fn and_then<T2>(self, parser: impl Parser<T2>) -> impl Parser<T2>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            self.parse(cursor, required)?;
            parser.parse(cursor, required)
        }
    }

    fn and<T2>(self, parser: impl Parser<T2>) -> impl Parser<(T, T2)>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let out_1 = self.parse(cursor, required)?;
            let out_2 = parser.parse(cursor, required)?;
            Ok((out_1, out_2))
        }
    }

    fn or(self, parser: impl Parser<T>) -> impl Parser<T>
    where
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let original_pos = cursor.pos;
            match self.parse(cursor, false) {
                Ok(succ) => Ok(succ),
                Err(Some(err)) => Err(Some(err)),
                Err(None) => {
                    cursor.pos = original_pos;
                    parser.parse(cursor, required)
                }
            }
        }
    }

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

    fn into<T2>(self) -> impl Parser<T2>
    where
        T2: From<T>,
        Self: Sized,
    {
        move |cursor: &mut Cursor, required: bool| {
            let out = self.parse(cursor, required)?;
            Ok(out.into())
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

/*========*
 * Choice *
 *========*/

fn alt<T>(label: &str, options: impl AltTuple<T>) -> impl Parser<T> {
    options.make_parser(label.to_owned())
}

trait AltTuple<T> {
    fn make_parser(self, label: String) -> impl Parser<T>;
}

macro_rules! define_alt {
    ($struct:ident, $type:ident, $( ($idx:tt, $parser:ident) ),*) => {
        struct $struct<$( $parser ),*>($( $parser ),*);

        impl<$type, $( $parser ),*> AltTuple<$type> for $struct<$( $parser ),*>
        where $( $parser : Parser<$type> ),* {
            fn make_parser(self, label: String) -> impl Parser<T> {
                move |cursor: &mut Cursor, required: bool| {
                    let original_pos = cursor.pos;
                    $(
                        match self.$idx.parse(cursor, false) {
                            Ok(succ) => return Ok(succ),
                            Err(Some(err)) => return Err(Some(err)),
                            Err(None) => cursor.pos = original_pos,
                        }
                    )*
                    if required {
                        Err(Some(ParseError {
                            message: format!("expected {label}"),
                            pos: cursor.pos,
                        }))
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
