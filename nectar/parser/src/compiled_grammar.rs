use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;

use table::{Item, Index, Table};
use fixity::*;
use lexer;
use lexer::Token;
use grammar;

use self::Part::*;


pub fn make_grammar(tokens: Vec<Token>, ops: Vec<grammar::Operator>)
                    -> CompiledGrammar {
    compile_grammar(grammar::Grammar::new(tokens, ops))
}


pub enum Part {
    HolePart,
    NamePart(Index<Token>),
    PostPart
}


pub struct Operator {
    pub name: String,
    pub fixity: Fixity,
    pub parts: Vec<Part>
}
impl<'g> HasPrec for Operator {
    fn prec(&self, side: Side) -> Prec {
        self.fixity.prec(side)
    }
}
impl<'g> fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}


pub struct CompiledGrammar {
    pub tokens: Table<Token>,
    ops: Table<Operator>,

    op_parts: HashSet<Index<Token>>,
    hint_table_init: HashMap<Index<Token>, Index<Operator>>,
    hint_table_next: HashMap<(Index<Operator>, Index<Token>), Index<Token>>
}
impl CompiledGrammar {
    pub fn is_op_part(&self, id: Index<Token>) -> bool {
        self.op_parts.contains(&id)
    }

    pub fn hint_init(&self, id: Index<Token>) -> Option<Index<Operator>> {
        self.hint_table_init.get(&id).map(|id| *id)
    }

    pub fn hint_next(&self, op_id: Index<Operator>, tok_id: Index<Token>)
                     -> Option<Index<Token>> {
        self.hint_table_next.get(&(op_id, tok_id)).map(|id| *id)
    }

    pub fn get_token(&self, id: Index<Token>) -> &Item<Token> {
        self.tokens.get(id)
    }

    pub fn get_op(&self, id: Index<Operator>) -> &Item<Operator> {
        self.ops.get(id)
    }
}


fn compile_grammar(grammar: grammar::Grammar) -> CompiledGrammar {

    let mut tokens = Table::new();
    let mut ops    = Table::new();
    let mut parts: HashMap<String, Index<Token>> = HashMap::new();

    {
        // TODO: Check that tokens aren't reused.
        for tok in grammar.tokens.into_iter() {
            tokens.add(tok);
        }

        let mut add_part = |word: String, req_space: bool| -> Index<Token> {
            // OPT: possible to do this with just &str?
            *parts.entry(word.clone()).or_insert_with(|| {
                let tok = lexer::part(&word, &word, req_space);
                let id = tokens.add(tok);
                id
            })
        };

        // TODO: check that there are no adjacent holes
        // TODO: check that every op begins with a name part
        for op in grammar.ops.into_iter() {
            let has_post = op.right_prec() != NO_PREC;
            let mut parts: Vec<Part> = op.parts.into_iter().map(|part| {
                match part {
                    grammar::Part::HolePart => HolePart,
                    grammar::Part::NamePart(name, req_space) => {
                        let index = add_part(name, req_space);
                        NamePart(index)
                    }
                }
            }).collect();
            if has_post {
                parts.push(PostPart)
            }

            ops.add(Operator{
                name: op.name,
                fixity: op.fixity,
                parts: parts
            });
        }
    }

    let mut hint_table_init = HashMap::new();
    let mut hint_table_next = HashMap::new();
    let mut op_parts = HashSet::new();
    {
        for op in ops.items.iter() {
            let init_part = op.parts.get(0).expect("compiled_grammar 1");
            match init_part {
                &NamePart(tok) => {
                    hint_table_init.insert(tok, op.index);
                }
                _ => panic!("compiled_grammar: every op must start with a name part")
            }

            let mut last_tok: Option<Index<Token>> = None;
            for part in op.parts.iter() {
                match part {
                    &HolePart => (),
                    &PostPart => (),
                    &NamePart(tok) => {
                        op_parts.insert(tok);
                        match last_tok {
                            None => (),
                            Some(last_tok) => {
                                let key = (op.index, last_tok);
                                hint_table_next.insert(key, tok);
                            }
                        }
                        last_tok = Some(tok);
                    }
                }
            }
        }
    }
    
    CompiledGrammar{
        tokens: tokens,
        ops: ops,
        op_parts: op_parts,
        hint_table_init: hint_table_init,
        hint_table_next: hint_table_next
    }
}
