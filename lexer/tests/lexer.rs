#[test]
fn test_lexer() {
    use lexer::{Lexeme, LexerBuilder, LEX_ERROR};

    fn show(lexeme: Option<Lexeme>) -> String {
        if let Some(lexeme) = lexeme {
            let token = if lexeme.token == LEX_ERROR {
                "ERR".to_owned()
            } else {
                format!("{}", lexeme.token)
            };
            format!(
                "{} {} {}:{}-{}:{}",
                token,
                lexeme.lexeme,
                lexeme.start.line,
                lexeme.start.col,
                lexeme.end.line,
                lexeme.end.col
            )
        } else {
            "None".to_owned()
        }
    }

    let mut builder = LexerBuilder::new(r#"[ \t\r\n]+"#).unwrap();
    builder.regex("[a-zA-Z_]+").unwrap();
    builder.string("raise").unwrap();
    builder.string("(").unwrap();
    builder.string("raise").unwrap();
    builder.string(")").unwrap();
    let lexer = builder.finish().unwrap();

    let source = "raised";
    let mut lexemes = lexer.lex(source);
    assert_eq!(show(lexemes.next()), "0 raised 0:0-0:6");
    assert_eq!(show(lexemes.next()), "None");

    let source = "raise(my_error)";
    let mut lexemes = lexer.lex(source);
    assert_eq!(show(lexemes.next()), "1 raise 0:0-0:5");
    assert_eq!(show(lexemes.next()), "2 ( 0:5-0:6");
    assert_eq!(show(lexemes.next()), "0 my_error 0:6-0:14");
    assert_eq!(show(lexemes.next()), "3 ) 0:14-0:15");
    assert_eq!(show(lexemes.next()), "None");

    let source = "x\n$$ !";
    let mut lexemes = lexer.lex(source);
    assert_eq!(show(lexemes.next()), "0 x 0:0-0:1");
    assert_eq!(show(lexemes.next()), "ERR $$ 1:0-1:2");
    assert_eq!(show(lexemes.next()), "ERR ! 1:3-1:4");
    assert_eq!(show(lexemes.next()), "None");
}

/* TODO: update
#[test]
fn test_lexer_without_line_col() {
    use lexer::lexer_without_line_col::{LexerBuilder, LEX_ERROR};

    let mut builder = LexerBuilder::new(r#"[ \t\r\n]+"#).unwrap();

    let tok_var = builder.regex("[a-zA-Z_]+").unwrap();
    let tok_lparen = builder.string("(").unwrap();
    let tok_raise = builder.string("raise").unwrap();
    let tok_rparen = builder.string(")").unwrap();

    let lexer = builder.finish().unwrap();

    let mut lexemes = lexer.lex("raised");
    assert_eq!(lexemes.next(), Some((tok_var, "raised")));
    assert_eq!(lexemes.next(), None);

    let mut lexemes = lexer.lex("raise(my_error)");
    assert_eq!(lexemes.next(), Some((tok_raise, "raise")));
    assert_eq!(lexemes.next(), Some((tok_lparen, "(")));
    assert_eq!(lexemes.next(), Some((tok_var, "my_error")));
    assert_eq!(lexemes.next(), Some((tok_rparen, ")")));
    assert_eq!(lexemes.next(), None);

    let mut lexemes = lexer.lex("x $$ !");
    assert_eq!(lexemes.next(), Some((tok_var, "x")));
    assert_eq!(lexemes.next(), Some((LEX_ERROR, "$$")));
    assert_eq!(lexemes.next(), Some((LEX_ERROR, "!")));
    assert_eq!(lexemes.next(), None);
}
*/
