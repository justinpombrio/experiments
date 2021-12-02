use compiled_grammar::CompiledGrammar;
use expr::Expr;
use parser::{Parser, ParseResult};


pub use compiled_grammar::make_grammar;
pub use lexer::{Token};
pub use grammar::*;
pub use source::SourceFile;


pub fn parse<'s, 'g>(grammar: &'g CompiledGrammar, source: &'s SourceFile)
                     -> ParseResult<'s, 'g, Expr<'s, 'g>> {
    let mut parser = Parser::new(grammar, source);
    parser.parse(source)
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let grammar = make_grammar(

            vec!(identifier("Id",         "^[a-z][_a-zA-Z0-9]*"),
                 identifier("Type Id",    "^[A-Z][_a-zA-Z0-9]*"),
                 constant(  "True",       "true"),
                 constant(  "False",      "false"),
                 literal(   "Nat",        "^[0-9]+"),
                 whitespace("Whitespace", "^[ \t\n]+"),
                 literal(   "String",     "^\"[^\"]*\"")),

            vec!(op("And",     infixr(60),  vec!(name("&&"))),
                 op("Or",      infixl(40),  vec!(name("||"))),
                 op("Ternary", infixr(20),  vec!(name("?"), hole, name(":"))),
                 op("Not",     prefix(80),  vec!(name("not"))),
                 op("Fact",    postfix(80), vec!(punct("!"))),
                 op("Bracket", postfix(70), vec!(punct("["), hole, punct("]"))),
                 op("If",      prefix(20),
                    vec!(name("if"),
                         hole,
                         name("then"),
                         hole,
                         name("else"))),
                 op("While", nilfix,
                    vec!(name("while"), hole,
                         name("do"), hole,
                         name("end")))));

        fn make_testfile(src: &str) -> SourceFile {
            SourceFile::new("Testfile".to_string(), src.to_string())
        }

        let test_parse = |src: &str| {
            let source = make_testfile(src);
            let result = parse(&grammar, &source);
            match result {
                Ok(result) => format!("{}", result),
                Err(err)   => {
                    println!("{}", err);
                    assert!(false);
                    panic!()
                }
            }
        };

        assert_eq!(test_parse(""), ".");
        assert_eq!(test_parse("33"), "33");
        assert_eq!(test_parse("true"), "true");
        assert_eq!(test_parse("wibble"), "wibble");
        assert_eq!(test_parse("  \"str\" "), "\"str\"");
        assert_eq!(test_parse("1 2 3"), "[Juxt 1 [Juxt 2 3]]");
        assert_eq!(test_parse("true && false"), "[And true false]");
        assert_eq!(test_parse("1 && 2 || 3"), "[Or [And 1 2] 3]");
        assert_eq!(test_parse("1 || 2 && 3"), "[Or 1 [And 2 3]]");
        assert_eq!(test_parse("1 && 2 && 3"), "[And 1 [And 2 3]]");
        assert_eq!(test_parse("1 || 2 || 3"), "[Or [Or 1 2] 3]");
        assert_eq!(test_parse("&&"), "[And . .]");
        assert_eq!(test_parse("1[2]"), "[Bracket 1 2]");
        assert_eq!(test_parse("[1]"), "[Bracket . 1]");
        assert_eq!(test_parse("1 ? 2 : 3"), "[Ternary 1 2 3]");
        assert_eq!(test_parse("1 ? 2 : 3 ? 4 : 5"), "[Ternary 1 2 [Ternary 3 4 5]]");
        assert_eq!(test_parse("1 ? 2 :"), "[Ternary 1 2 .]");
    }
}
