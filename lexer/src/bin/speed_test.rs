use lexer::{Lexer, LexerBuilder};
use std::fs::read_to_string;

fn json_lexer() -> Lexer {
    let mut builder = LexerBuilder::new(r#"[ \t\r\n]+"#).unwrap();

    // Strings
    builder.regex(r#""([^\\"]|(\\.))*""#).unwrap();
    // Numbers
    builder
        .regex(r#"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?"#)
        .unwrap();
    // Everything else
    builder.string("null").unwrap();
    builder.string("true").unwrap();
    builder.string("false").unwrap();
    builder.string("[").unwrap();
    builder.string("]").unwrap();
    builder.string(",").unwrap();
    builder.string("{").unwrap();
    builder.string("}").unwrap();
    builder.string(":").unwrap();

    builder.finish().unwrap()
}

fn main() {
    let lexer = json_lexer();
    let source = &read_to_string("sample.json").unwrap();
    // On laptop --release:
    // - w/o line&col: 23s
    // - w/ line&col:  27s
    // - w/ indexer:   28s
    for _ in 0..5000 {
        let lexemes = lexer.lex(source);
        let count = lexemes.count();
        assert_eq!(count, 14517);
    }
}
