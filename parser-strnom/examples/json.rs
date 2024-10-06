use strnom::{alt, parse, regex, string, Parser};

// A simple JSON parser. Does not handle things like string escapes or numbers with
// exponents in them.

// cat examples/sample.json | cargo run --release --example json

#[derive(Debug, Clone)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

fn main() {
    println!("ok")
}
