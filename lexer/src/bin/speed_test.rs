use lexer::lexer_without_line_col::{Lexer, LexerBuilder};
use lexer::line_and_col_indexer::LineAndColIndexer;
use lexer::{Lexer as LexerLC, LexerBuilder as LexerBuilderLC};
use std::fs::read_to_string;
use std::time::Instant;

fn json_lexer_lc() -> LexerLC {
    let mut builder = LexerBuilderLC::new(r#"[ \t\r\n]+"#).unwrap();

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
    let lexer_lc = json_lexer_lc();
    let lexer = json_lexer();
    let source = &read_to_string("sample.json").unwrap();
    // On laptop --release:
    // Lexer w/o line&col:  4874ms
    // Lexer with line&col: 5636ms (1.1563398 longer)
    // Lexer w indexer:     7098ms (1.2594038 longer)

    let now = Instant::now();
    for _ in 0..1000 {
        let lexemes = lexer.lex(source);
        let count = lexemes.count();
        assert_eq!(count, 14517);
    }
    let elapsed = now.elapsed().as_millis();
    println!("Lexer w/o line&col:  {}ms", elapsed);

    let now = Instant::now();
    for _ in 0..1000 {
        let lexemes = lexer_lc.lex(source);
        let mut count = 0;
        for lexeme in lexemes {
            count += lexeme.start.col;
            count += lexeme.end.utf8_col;
        }
        assert_eq!(count, 1061040);
    }
    let elapsed_lc = now.elapsed().as_millis();
    println!(
        "Lexer with line&col: {}ms ({} longer)",
        elapsed_lc,
        elapsed_lc as f32 / elapsed as f32
    );

    let now = Instant::now();
    for _ in 0..1000 {
        let lexemes = lexer.lex(source);
        let indexer = LineAndColIndexer::new(source);
        let mut count = 0;
        for lexeme in lexemes {
            count += indexer.start_col(lexeme.1);
            count += indexer.end_utf8_col(lexeme.1);
        }
        assert_eq!(count, 1061040);
    }
    let elapsed_index = now.elapsed().as_millis();
    println!(
        "Lexer w indexer:     {}ms ({} longer)",
        elapsed_index,
        elapsed_index as f32 / elapsed_lc as f32
    );
}
