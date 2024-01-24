//! From https://github.com/rust-bakery/nom/blob/main/examples/json.rs
//! with comments removed

use crate::{Json, JsonParser};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::char,
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ParseError, VerboseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult, Parser,
};
use std::str;

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false)).parse(input)
}

fn null<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value((), tag("null")).parse(input)
}

fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(take_until("\""), char('\"')))),
    )(i)
}

fn array<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<Json>, E> {
    context(
        "array",
        preceded(
            char('['),
            cut(terminated(
                separated_list0(preceded(sp, char(',')), json_value),
                preceded(sp, char(']')),
            )),
        ),
    )(i)
}

fn key_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, Json), E> {
    separated_pair(
        preceded(sp, string),
        cut(preceded(sp, char(':'))),
        json_value,
    )
    .parse(i)
}

fn hash<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<(String, Json)>, E> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                map(
                    separated_list0(preceded(sp, char(',')), key_value),
                    |tuple_vec| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                preceded(sp, char('}')),
            )),
        ),
    )(i)
}

fn json_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Json, E> {
    preceded(
        sp,
        alt((
            map(hash, Json::Object),
            map(array, Json::Array),
            map(string, |s| Json::String(String::from(s))),
            map(double, Json::Number),
            map(boolean, Json::Boolean),
            map(null, |_| Json::Null),
        )),
    )
    .parse(i)
}

fn root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Json, E> {
    delimited(
        sp,
        alt((
            map(hash, Json::Object),
            map(array, Json::Array),
            map(null, |_| Json::Null),
        )),
        opt(sp),
    )
    .parse(i)
}

pub struct NomParser;

impl JsonParser for NomParser {
    fn new() -> NomParser {
        NomParser
    }

    fn name(&self) -> &'static str {
        "nom"
    }

    fn parse_json(&self, input: &str) -> Json {
        match root::<VerboseError<&str>>(input) {
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                panic!(
                    "verbose errors - `root::<VerboseError>(data)`:\n{}",
                    convert_error(input, e)
                );
            }
            Err(Err::Incomplete(_)) => todo!(),
            Ok((_, json)) => json,
        }
    }
}

/*
fn main() {
  let data = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ] ,
  \"c\": { \"hello\" : \"world\"
  }
  } ";

  println!(
    "will try to parse valid JSON data:\n\n**********\n{}\n**********\n",
    data
  );

  println!(
    "parsing a valid file:\n{:#?}\n",
    root::<(&str, ErrorKind)>(data)
  );

  let data = "  { \"a\"\t: 42,
  \"b\": [ \"x\", \"y\", 12 ] ,
  \"c\": { 1\"hello\" : \"world\"
  }
  } ";

  println!(
    "will try to parse invalid JSON data:\n\n**********\n{}\n**********\n",
    data
  );

    "basic errors - `root::<(&str, ErrorKind)>(data)`:\n{:#?}\n",
    root::<(&str, ErrorKind)>(data)
  );

  println!("parsed verbose: {:#?}", root::<VerboseError<&str>>(data));

  match root::<VerboseError<&str>>(data) {
    Err(Err::Error(e)) | Err(Err::Failure(e)) => {
      println!(
        "verbose errors - `root::<VerboseError>(data)`:\n{}",
        convert_error(data, e)
      );
    }
    _ => {}
  }

  assert!(root::<(&str, ErrorKind)>("null").is_ok());
}
*/
