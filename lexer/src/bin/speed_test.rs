use lexer::indexing_lexer::{Lexer as LexerI, LexerBuilder as LexerBuilderI};
use lexer::lexer_without_line_col::{Lexer, LexerBuilder};
use lexer::line_and_col_indexer::LineAndColIndexer;
use lexer::{Lexer as LexerLC, LexerBuilder as LexerBuilderLC};
use std::fs::read_to_string;
use std::time::Instant;

// On laptop --release:
//   Lexer w/o line&col:  6083ms
//   Lexer with line&col: 7021ms (1.1542002 longer)
//   Indexing lexer:      7294ms (1.0388833 longer)
//   Lexer w/ indexer:    7622ms (1.0856004 longer)

// Earlier on laptop --release:
//   Lexer w/o line&col:  4874ms
//   Lexer with line&col: 5636ms (1.1563398 longer)
//   Lexer w/ indexer:    7098ms (1.2594038 longer)

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

fn json_lexer_no_lc() -> Lexer {
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

fn json_lexer_i() -> LexerI {
    let mut builder = LexerBuilderI::new(r#"[ \t\r\n]+"#).unwrap();

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
    let lexer_no_lc = json_lexer_no_lc();
    let lexer_lc = json_lexer_lc();
    let lexer_i = json_lexer_i();
    let source = &read_to_string("sample.json").unwrap();

    let now = Instant::now();
    for _ in 0..1000 {
        let lexemes = lexer_no_lc.lex(source);
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
            count += lexeme.end.line;
            count += lexeme.end.utf8_col;
        }
        assert_eq!(count, 31156367);
    }
    let elapsed_lc = now.elapsed().as_millis();
    println!(
        "Lexer with line&col: {}ms ({} longer)",
        elapsed_lc,
        elapsed_lc as f32 / elapsed as f32
    );

    let now = Instant::now();
    for _ in 0..1000 {
        let mut lexemes = lexer_i.lex(source);
        let spans = (&mut lexemes).map(|(_, span)| span).collect::<Vec<_>>();
        let indexer = lexemes.into_indexer();
        let mut count = 0;
        for span in spans {
            count += indexer.start_col(span);
            count += indexer.end_line(span);
            count += indexer.end_utf8_col(span);
        }
        assert_eq!(count, 31156367);
    }
    let elapsed_index = now.elapsed().as_millis();
    println!(
        "Indexing lexer:      {}ms ({} longer)",
        elapsed_index,
        elapsed_index as f32 / elapsed as f32
    );

    let now = Instant::now();
    for _ in 0..1000 {
        let mut lexemes = lexer_i.lex(source);
        let spans = (&mut lexemes).map(|(_, span)| span).collect::<Vec<_>>();
        let indexer = LineAndColIndexer::new(source);
        let mut count = 0;
        for span in spans {
            count += indexer.start_col(span);
            count += indexer.end_line(span);
            count += indexer.end_utf8_col(span);
        }
        assert_eq!(count, 31156367);
    }
    let elapsed_indexer = now.elapsed().as_millis();
    println!(
        "Lexer w/ indexer:    {}ms ({} longer)",
        elapsed_indexer,
        elapsed_indexer as f32 / elapsed as f32
    );
}
