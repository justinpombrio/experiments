use crate::{Json, JsonParser};
use parser_ll1::{
    choice, tuple, CompiledParser, Grammar, GrammarError, ParseError, Parser, Recursive,
};

pub struct LL1Parser(Box<dyn Fn(&str, &str) -> Result<Json, ParseError>>);

impl JsonParser for LL1Parser {
    fn new() -> LL1Parser {
        let compiled_parser = make_json_parse_fn().unwrap();
        LL1Parser(Box::new(move |filename: &str, input: &str| {
            compiled_parser.parse(filename, input)
        }))
    }

    fn name(&self) -> &'static str {
        "ll1"
    }

    fn parse_json(&self, input: &str) -> Json {
        match (self.0)("stdin", input) {
            Ok(json) => json,
            Err(err) => panic!("{}", err),
        }
    }
}

fn make_json_parse_fn() -> Result<impl CompiledParser<Json>, GrammarError> {
    use std::str::FromStr;

    let mut g = Grammar::with_whitespace("[ \t\r\n]+")?;

    let json_p = Recursive::new("json value");

    // Null
    let null_p = g.string("null")?.constant(Json::Null);

    // Bools
    let true_p = g.string("true")?.constant(Json::Boolean(true));
    let false_p = g.string("false")?.constant(Json::Boolean(false));
    let bool_p = choice("boolean", (true_p, false_p));

    // Numbers. This is a bad regex that only works for some numbers
    let number_p = g
        .regex("number", "[1-9][0-9]*(\\.[0-9]*)?|\\.[0-9]*")?
        .try_span(|s| f64::from_str(s.substr))
        .map(Json::Number);

    // Strings. Not implementing Json string escapes for this small test case.
    let plain_string_p = g
        .regex("string", r#""([^"\\]|\\.)*""#)?
        .span(|span| span.substr[1..span.substr.len() - 1].to_owned());
    let string_p = plain_string_p.clone().map(Json::String);

    // Arrays
    let array_elems_p = json_p.refn().many_sep0(g.string(",")?);
    let array_p = tuple("array", (g.string("[")?, array_elems_p, g.string("]")?))
        .map(|(_, elems, _)| Json::Array(elems));

    // Objects
    let entry_p = tuple(
        "dictionary entry",
        (plain_string_p, g.string(":")?, json_p.refn()),
    )
    .map(|(key, _, val)| (key, val));
    let entries_p = entry_p.many_sep0(g.string(",")?);
    let dict_p = tuple("dictionary", (g.string("{")?, entries_p, g.string("}")?))
        .map(|(_, entries, _)| Json::Object(entries));

    let json_p = json_p.define(choice(
        "json value",
        (null_p, bool_p, number_p, string_p, array_p, dict_p),
    ));

    g.compile_parser(json_p)
}
