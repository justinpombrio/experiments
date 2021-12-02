use std::fmt;
use std::cmp;
use std::hash;
use std::collections::HashMap;
use regex::Regex;

use token::*;
use operator::*;
use catalog::{Index, Indexed, Catalog};


/****************************** PUBLIC ******************************/


pub fn make_grammar(name: &str, tokens: Vec<Token>, ops: Vec<OperatorSpec>) -> Grammar {
    Grammar::new(name, tokens, ops)
}


/****************************** PRIVATE ******************************/


pub struct Grammar {
    name: String,
    tokens: Catalog<Token>,
    ops: Catalog<Operator>,
    part_table: HashMap<String, Index<Token>>
}

impl Grammar {
    pub fn new(name: &str, tokens: Vec<Token>, mut ops: Vec<OperatorSpec>)
           -> Grammar {
        let mut g = Grammar{
            name: name.to_string(),
            tokens: Catalog::new(),
            ops: Catalog::new(),
            part_table: HashMap::new()
        };

        for token in tokens.into_iter() {
            g.add_token(token);
        }
        for op in ops.into_iter() {
            g.add_op(op)
        }
        // Insert magic "Program" operator
        g.add_op(op("Program", vec!(magic_token("SOF"), magic_token("EOF"))));
        g
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.add(token);
    }

    fn add_op(&mut self, op: OperatorSpec) {
        let mut new_op = Operator{
            name: op.name,
            parts: vec!()
        };
        for part in op.parts.into_iter() {
            let part_index = self.add_part(part);
            new_op.parts.push(part_index);
        }
        self.ops.add(new_op);
    }

    fn add_part(&mut self, part: Token) -> Index<Token> {
        match self.part_table.get(&part.name) {
            None => {
                let token_id = self.tokens.add(part.clone());
                self.part_table.insert(part.name, token_id);
                token_id
            }
            Some(&token_id) => {
                token_id
            }
        }
    }

    pub fn op_table(&self) -> HashMap<Index<Token>, (&Operator, Vec<Indexed<Token>>)> {
        let mut table = HashMap::new();
        for op in self.ops.iter() {
            let token = op.parts[0];
            let mut parts = vec!();
            for &part in op.parts[1..].iter() {
                parts.push(self.lookup_token(part));
            }
            table.insert(token, (op.data, parts));
        }
        table
    }

    pub fn compile(&self) -> CompiledGrammar {
        CompiledGrammar{
            grammar: self,
            op_table: self.op_table()
        }
    }

    pub fn token_table(&self) -> &Catalog<Token> {
        &self.tokens
    }

    pub fn lookup_token(&self, token: Index<Token>) -> Indexed<Token> {
        self.tokens.lookup(token)
    }

    pub fn start_token<'g>(&'g self) -> Indexed<'g, Token> {
        // TODO: Efficiency
        for token in self.tokens.iter() {
            if token.name == "SOF" {
                return token
            }
        }
        panic!("Grammar: SOF token not found")
    }

    pub fn end_token<'g>(&'g self) -> Indexed<'g, Token> {
        // TODO: Efficiency
        for token in self.tokens.iter() {
            if token.name == "EOF" {
                return token
            }
        }
        panic!("Grammar: EOF token not found")
    }
}
/*
fn whatever() {
    let grammar = Grammar::new("G", vec!(), vec!());
    let grammar = grammar.compile();
    println!("ok");
}
*/

pub struct CompiledGrammar<'g> {
    grammar: &'g Grammar,
    op_table: HashMap<Index<Token>, (&'g Operator, Vec<Indexed<'g, Token>>)>
}
/*
impl<'s, 'g> Lexeme<'s, 'g> {
    pub fn lexeme_start(src: &'s SourceFile, grammar: &'g Grammar) -> Lexeme<'s, 'g> {
        Lexeme{
            span: src.start(),
            token: grammar.start_token()
        }
    }

    pub fn lexeme_end(src: &'s SourceFile, grammar: &'g Grammar) -> Lexeme<'s, 'g> {
        Lexeme{
            span: src.end(),
            token: grammar.end_token()
        }
    }
}
*/
