use std::fmt;

use catalog::{Index, Indexed};
use token::*;


/****************************** PUBLIC ******************************/

pub fn op(name: &str, parts: Vec<Token>) -> OperatorSpec {
    OperatorSpec{
        name: name.to_string(),
        parts: parts
    }
}


/****************************** PRIVATE ******************************/


pub struct OperatorSpec {
    pub name: String,
    pub parts: Vec<Token>
}

pub struct Operator {
    pub name: String,
    pub parts: Vec<Index<Token>>
}

pub struct CompiledOperator<'g> {
    name: String,
    parts: Vec<Indexed<'g, Token>>
}

impl<'g> fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
   }
}
