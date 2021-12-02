use fixity::Fixity;
use fixity::Fixity::*;
use fixity::Prec;
use fixity::HasPrec;
use fixity::Side;
use lexer;
use lexer::Token;



// Tokens //
    
pub fn whitespace(name: &str, regex: &str) -> Token {
    lexer::whitespace(name, regex)
}

pub fn identifier(name: &str, regex: &str) -> Token {
    lexer::identifier(name, regex)
}

pub fn literal(name: &str, regex: &str) -> Token {
    lexer::literal(name, regex)
}

pub fn constant(name: &str, word: &str) -> Token {
    lexer::constant(name, word)
}


// Fixity //                        

pub use fixity::NO_PREC;
pub use fixity::MIN_PREC;

pub fn infix(lprec: u16, rprec: u16) -> Fixity {
    Infix(Prec::new(lprec), Prec::new(rprec))
}

pub fn infixl(prec: u16) -> Fixity {
    Infix(Prec{ prec: prec, decimal: 0 },
          Prec{ prec: prec, decimal: 1 })
}

pub fn infixr(prec: u16) -> Fixity {
    Infix(Prec{ prec: prec, decimal: 1 },
          Prec{ prec: prec, decimal: 0 })
}

pub fn prefix(prec: u16) -> Fixity {
    Prefix(Prec::new(prec))
}

pub fn postfix(prec: u16) -> Fixity {
    Postfix(Prec::new(prec))
}

#[allow(non_upper_case_globals)]
pub const nilfix: Fixity = Nilfix;


// Operators //

pub enum Part {
    HolePart,
    NamePart(String, bool) // lexeme, req_space
}

pub fn name(lexeme: &str) -> Part {
    Part::NamePart(lexeme.to_string(), true)
}

pub fn punct(lexeme: &str) -> Part {
    Part::NamePart(lexeme.to_string(), false)
}

pub fn op(name: &str, fixity: Fixity, parts: Vec<Part>) -> Operator {
    Operator{
        name: name.to_string(),
        fixity: fixity,
        parts: parts
    }
}

#[allow(non_upper_case_globals)]
pub const hole: Part = Part::HolePart;

pub struct Operator {
    pub name: String,
    pub fixity: Fixity,
    pub parts: Vec<Part>
}
impl HasPrec for Operator {
    fn prec(&self, side: Side) -> Prec {
        self.fixity.prec(side)
    }
}




// Grammar //

pub struct Grammar {
    pub tokens: Vec<Token>,
    pub ops: Vec<Operator>
}
impl Grammar {
    pub fn new(tokens: Vec<Token>, ops: Vec<Operator>) -> Grammar {
        Grammar{
            tokens: tokens,
            ops: ops
        }
    }
}
