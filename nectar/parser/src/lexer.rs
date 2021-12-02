use regex::Regex;

use source::{SourceFile, Pos, Span, Spanned};
use error::{ParseError, lexing_error};
use token::{Pattern, Token, TokenSet};
use item::{Lexeme};


/****************************** PUBLIC ******************************/


pub fn lex<'s, 'g>(source: &'s SourceFile, grammar: TokenSet<'g>)
                   -> Result<Vec<Lexeme<'s, 'g>>, ParseError<'s, 'g>> {
    let mut lexer = Lexer::new(grammar, source);
    let mut lexemes = vec!();
    while !lexer.at_eof() {
        match lexer.consume_lexeme() {
            Some(lex) => lexemes.push(lex),
            None => {
                return Err(lexing_error(lexer.current_span()))
            }
        }
    }
    Ok(lexemes)
}


/****************************** PRIVATE ******************************/


struct Match<'s, 'g> {
    lexeme: Lexeme<'s, 'g>,
    spaced: bool,
    ignore: bool,
    len: usize
}

struct Lexer<'s, 'g> {
    source: &'s SourceFile,
    grammar: TokenSet<'g>,
    ptr: &'s str,
    pos: Pos,
    spaced: bool
}

impl<'s, 'g> Lexer<'s, 'g> {

    fn new(grammar: TokenSet<'g>, source: &'s SourceFile)
           -> Lexer<'s, 'g> {
        Lexer{
            source: source,
            grammar: grammar,
            ptr: &source.text,
            pos: Pos::new(0),
            spaced: true
        }
    }

    fn at_eof(&self) -> bool {
        self.ptr.is_empty()
    }

    fn current_span(&self) -> Span<'s> {
        Span::new(self.source, self.pos, self.pos + 1)
    }

    fn match_span(&self, len: usize) -> Span<'s> {
        Span::new(self.source, self.pos, self.pos + len)
    }

    fn match_string(&self, s: &str) -> Option<Span<'s>> {
        if self.ptr.starts_with(s) {
            Some(self.match_span(s.len()))
        } else {
            None
        }
    }

    fn match_regex(&self, regex: &Regex) -> Option<Span<'s>> {
        regex.find(self.ptr).map(|(i, j)| {
            if i != 0 {
                panic!("Internal error: regex match not at start of str.")
            }
            self.match_span(j)
        })
    }

    fn match_pattern(&self, pattern: &Pattern, req_space: bool)
                     -> Option<Span<'s>> {
        if self.spaced || !req_space {
            match pattern {
                &Pattern::String(ref s) => self.match_string(s),
                &Pattern::Regex(ref re) => self.match_regex(re),
                &Pattern::None          => None
            }
        } else {
            None
        }
    }

    fn match_token(&self, token: &Token) -> Option<Span<'s>> {
        self.match_pattern(&token.pattern, token.req_space)
    }

    fn match_lex(&self) -> Option<Match<'s, 'g>> {
        let mut best_match: Option<Match<'s, 'g>> = None;
        for token in self.grammar.iter() {
            self.match_token(&token).map(|span| {
                // TODO: Prefer keyword matches? Messes with `-3`.
                
                // Prefer longer matches (and later matches)
                let is_better = match &best_match {
                    &None        => true,
                    &Some(ref m) => span.len() >= m.len
                };
                
                if is_better {
                    best_match = Some(Match{
                        lexeme: Spanned::new(span, token),
                        ignore: token.ignore,
                        spaced: !token.req_space,
                        len: span.len()
                    });
                }
            });
        }
        best_match
    }

    fn consume(&mut self, span: Span) {
        let len = span.len();
        self.ptr = &self.ptr[len ..];
        self.pos = self.pos + len;
    }

    fn consume_lexeme(&mut self) -> Option<Lexeme<'s, 'g>> {
        loop {
            match self.match_lex() {
                Some(m) => {
                    self.consume(m.lexeme.span);
                    self.spaced = m.spaced;
                    if !m.ignore {
                        return Some(m.lexeme)
                    }
                }
                None => return None
            }
        }
    }
}

/*
impl<'s, 'g> Iterator for Lexer<'s, 'g> {
    type Item = Result<Lexeme<'s, 'g>, ParseError<'s, 'g>>;
    fn next(&mut self) -> Option<Result<Lexeme<'s, 'g>, ParseError<'s, 'g>>> {
        if self.at_eof() {
            None
        } else {
            match self.consume_lexeme() {
                Some(lex) => Some(Ok(lex)),
                None => {
                    let span = Span::new(self.source, self.pos, self.pos + 1);
                    Some(Err(lexing_error(span)))
                }
            }
        }
    }
}
*/




/****************************** TESTS ******************************/

#[cfg(test)]
mod test {
    use source::SourceFile;
    use super::*;
    use token::*;
    use item::Lexeme;
    use catalog::*;

    #[test]
    fn test_tokenize() {
        let mut g: Catalog<Token> = Catalog::new();

        let t_id       = g.add(identifier("Id", "^[a-z][_a-zA-Z0-9]*"));
        let t_var      = g.add(identifier("Var", "^\\$[_a-zA-Z0-9-]+"));
        let t_type_id  = g.add(identifier("TypeId", "^[A-Z][_a-zA-Z0-9]*"));
        let t_true     = g.add(constant("True", "true"));
        let t_false    = g.add(constant("False", "false"));
        let t_num      = g.add(literal("Num", "^[0-9]+"));
                         g.add(whitespace("Space", "^[ \t\n]+"));
        let t_str      = g.add(literal("Str", "^\"[^\"]*\""));
                         g.add(punctuation("+"));
        let t_plusplus = g.add(punctuation("++"));
        let t_comma    = g.add(punctuation(","));
                         g.add(keyword("begin"));
        let t_end      = g.add(keyword("end"));

        let source: SourceFile = SourceFile::new(
            "TESTFILE".to_string(),
            "\"str\" \tfoo2,++$a  Bar 33 true end false".to_string());
        let lexemes: Result<Vec<Lexeme>, _> = lex(&source, &g);
        let lexemes = match lexemes {
            Err(err) => panic!("Failed to lex: {}", err),
            Ok(lexemes) => lexemes
        };
        let lexeme_strs: Vec<String> = lexemes.iter().map(|lex| {
            format!("{}", lex)
        }).collect();
        let tokens: Vec<Index<Token>> = lexemes.iter().map(|lex| {
            lex.index()
        }).collect();

        assert_eq!(lexeme_strs,
                   vec!("\"str\"", "foo2", ",", "++", "$a", "Bar",
                        "33", "true", "end", "false"));
        assert_eq!(tokens,
                   vec!(t_str, t_id, t_comma, t_plusplus, t_var, t_type_id,
                        t_num, t_true, t_end, t_false));
    }
}
