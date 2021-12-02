use std::fmt;

use table::*;
use source::*;
use compiled_grammar::*;
use compiled_grammar::Part::*;
use lexer::*;
use lexer::TokenKind::*;
use fixity::*;
use expr::*;
use expr::Expr::*;
use error::*;
use util::display_sep;


pub struct Parser<'s, 'g> {
    source: &'s SourceFile,
    grammar: &'g CompiledGrammar,
    lexemes: Vec<Lexeme<'s, 'g>>,
    anticipations: Vec<Option<&'g Item<Token>>>
}

impl<'s, 'g> fmt::Display for Parser<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{{"));
        try!(display_sep(f, " ", self.lexemes.iter().rev()));
        write!(f, "}}")
    }
}

pub type ParseResult<'s, 'g, T> = Result<T, ParseError<'s, 'g>>;

impl<'s, 'g> Parser<'s, 'g> {
    
    pub fn new(grammar: &'g CompiledGrammar, source: &'s SourceFile)
           -> Parser<'s, 'g> {
        Parser{
            source: source,
            grammar: grammar,
            lexemes: vec!(),
            anticipations: vec!()
        }
    }

    pub fn parse(&mut self, source: &'s SourceFile)
                 -> ParseResult<'s, 'g, Expr<'s, 'g>> {
        let mut lexemes = match lex(source, &self.grammar.tokens) {
            Ok(lexemes) => lexemes,
            Err(span)   => return Err(tokenization_error(span))
        };
        lexemes.reverse();
        self.source = source;
        self.lexemes = lexemes;
        self.parse_expr(MIN_PREC)
    }

    fn anticipating(&self, lexeme: Lexeme<'s, 'g>) -> bool {
        for anticipation in self.anticipations.iter() {
            if Some(lexeme.token) == *anticipation {
                return true;
            }
        }
        false
    }

    fn get_op(&self, lex: Lexeme<'s, 'g>) -> Option<&'g Item<Operator>> {
        self.grammar.hint_init(lex.token.index).map(|op_id| {
            self.grammar.get_op(op_id)
        })
    }

    fn get_anticipation(&self, op: &Item<Operator>, tok: &Item<Token>)
                        -> Option<&'g Item<Token>> {
        self.grammar.hint_next(op.index, tok.index).map(|tok_id| {
            self.grammar.get_token(tok_id)
        })
    }

//    fn is_op_part(&self, lex: Lexeme<'s, 'g>) -> bool {
//        self.grammar.is_op_part(lex.token.index)
//    }

    fn next_prec(&self) -> Prec {
        match self.lexemes.last() {
            None => MIN_PREC,
            Some(&lex) => {
                match lex.token.kind {
                    Whitespace => panic!("parser: unexpected whitespace"),
                    Identifier | Constant | Literal => NO_PREC,
                    OpPart(_) =>
                        if self.anticipating(lex) {
                            MIN_PREC
                        } else {
                            match self.get_op(lex) {
                                None     => MIN_PREC,
                                Some(op) => op.left_prec()
                            }
                        }
                }
            }
        }
    }

    fn empty_span(&self) -> Span<'s> {
        match self.lexemes.last() {
            None      => self.source.end(),
            Some(lex) => (*lex).span.start()
        }
    }

    fn parse_expr(&mut self, prec: Prec)
                  -> ParseResult<'s, 'g, Expr<'s, 'g>> {
        let expr = try!(if self.next_prec() == NO_PREC {
            let lex = match self.lexemes.pop() {
                None      => return Ok(EmptyExpr(self.empty_span())),
                Some(lex) => lex
            };
            match lex.token.kind {
                Identifier | Constant | Literal => Ok(LexExpr(lex)),
                OpPart(_) =>
                    match self.get_op(lex) {
                        None     => Err(unexpected_part_error(lex)),
                        Some(op) => self.parse_op(op, lex, vec!())
                    },
                Whitespace => panic!("parser: unexpected whitespace")
            }
        } else {
            Ok(EmptyExpr(self.empty_span()))
        });
        self.parse_suffix(expr, prec)
    }

    fn parse_op(&mut self, op: &'g Item<Operator>, lex: Lexeme<'s, 'g>,
                mut args: Vec<Expr<'s, 'g>>)
        -> ParseResult<'s, 'g, Expr<'s, 'g>>
    {
        // Let the op parse its own first Part.
        self.lexemes.push(lex);
        // Parse the parts
        let part_args = try!(self.parse_parts(op, &op.parts, lex));
        for arg in part_args.into_iter() {
            args.push(arg);
        }
        // Done
        Ok(StxExpr(op, args))
    }

    fn parse_parts(&mut self, op: &'g Item<Operator>, parts: &Vec<Part>,
                   lex: Lexeme<'s, 'g>)
                   -> ParseResult<'s, 'g, Vec<Expr<'s, 'g>>> {
        let mut args = vec!();
        for part in parts.iter() {
            let anticipation = self.get_anticipation(op, lex.token);
            self.anticipations.push(anticipation);
            match try!(self.parse_part(op, &part)) {
                None      => { }
                Some(arg) => { args.push(arg) }
            }
            self.anticipations.pop();
        }
        Ok(args)
    }

    fn parse_part(&mut self, op: &'g Item<Operator>, part: &Part)
                  -> ParseResult<'s, 'g, Option<Expr<'s, 'g>>> {
        match part {
            &HolePart => Ok(Some(try!(self.parse_expr(MIN_PREC)))),
            &PostPart => {
                Ok(Some(try!(self.parse_expr(op.right_prec()))))
            }
            &NamePart(expected_tok_id) => {
                let expected_tok = self.grammar.get_token(expected_tok_id);
                // Why is ^ not a reference?
                match self.lexemes.pop() {
                    None => {
                        Err(missing_part_error(self.source.end(), op, &expected_tok.data))
                    }
                    Some(lex) => {
                        if lex.token == expected_tok {
                            Ok(None)
                        } else {
                            Err(wrong_part_error(lex, &expected_tok.data))
                        }
                    }
                }
            }
        }
    }

    fn parse_suffix(&mut self, expr: Expr<'s, 'g>, prec: Prec)
                    -> ParseResult<'s, 'g, Expr<'s, 'g>> {
        let next_prec = self.next_prec();
        if next_prec == NO_PREC {
            if prec == MIN_PREC {
                let next_expr = try!(self.parse_expr(MIN_PREC));
                self.parse_suffix(Expr::juxt(expr, next_expr), prec)
            } else {
                Ok(expr)
            }
        } else if next_prec > prec {
            // NO_PREC > next_prec > prec --> next lexeme is an op
            let lex = self.lexemes.pop().expect("Parser::parse_suffix 1");
            let op = self.get_op(lex).expect("Parser::parse_suffix 2");
            let expr = try!(self.parse_op(op, lex, vec!(expr)));
            self.parse_suffix(expr, prec)
        } else {
            Ok(expr)
        }
    }
}
