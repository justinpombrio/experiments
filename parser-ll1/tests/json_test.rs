use parser_ll1::{
    choice, empty, recur, seq2, seq3, seq4, Grammar, GrammarError, ParseError, Parser,
};

#[derive(Debug, Clone)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn make_json_parser() -> Result<impl Fn(&str) -> Result<Json, ParseError>, GrammarError> {
    use std::str::FromStr;

    let mut g = Grammar::new("[ \t\r\n]+")?;
    let json_p = recur("json", |json_p| {
        // Null
        let null_p = g.string("null")?.value(Json::Null);

        // Bools
        let true_p = g.string("true")?.value(Json::Bool(true));
        let false_p = g.string("false")?.value(Json::Bool(false));
        let bool_p = choice("boolean", [true_p, false_p])?;

        // Numbers. This is a bad regex that only works for some numbers
        let number_p = g
            .regex("number", "[1-9][0-9]*(\\.[0-9]*)?")?
            .try_span(|s| f64::from_str(s.substr).map_err(|e| e.to_string()))
            .map(Json::Number);

        // Strings. Not implementing Json string escapes for this small test case.
        let plain_string_p = g
            .regex("string", r#""([^"\\]|\\.)*""#)?
            .span(|span| span.substr.to_owned());
        let string_p = plain_string_p.clone().map(Json::String);

        // Arrays
        let array_elems_p = json_p.clone().sep(g.string(",")?)?;
        let array_p = seq3(g.string("[")?, array_elems_p, g.string("]")?)?
            .map(|(_, elems, _)| Json::Array(elems));

        // Objects
        let entry_p =
            seq3(plain_string_p, g.string(":")?, json_p.clone())?.map(|(key, _, val)| (key, val));
        let entries_p = entry_p.sep(g.string(",")?)?;
        let dict_p = seq3(g.string("{")?, entries_p, g.string("}")?)?
            .map(|(_, entries, _)| Json::Object(entries));

        choice(
            "json value",
            [null_p, bool_p, number_p, string_p, array_p, dict_p],
        )
    })?;

    Ok(g.make_parse_fn(json_p))
}

#[test]
fn test_json() {
    let parser = make_json_parser().unwrap();
}
