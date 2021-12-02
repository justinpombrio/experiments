use std::fmt;

use self::Side::{Left, Right};
use self::Fixity::*;


pub static NO_PREC:  Prec = Prec{ prec: 200, decimal: 0 };
pub static MIN_PREC: Prec = Prec{ prec: 0,   decimal: 0 };

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prec {
    pub prec: u16,   // The actual precedence
    pub decimal: u16 // Minor adjustments for associativity
}
impl Prec {
    pub fn new(prec: u16) -> Prec {
        Prec{ prec: prec, decimal: 0 }
    }
}
impl fmt::Display for Prec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.prec, self.decimal)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side { Left, Right }

pub trait HasPrec {
    fn prec(&self, side: Side) -> Prec;

    fn left_prec(&self) -> Prec {
        self.prec(Left)
    }
    fn right_prec(&self) -> Prec {
        self.prec(Right)
    }
}

#[derive(Clone, Copy)]
pub enum Fixity {
    Nilfix,
    Prefix(Prec),
    Postfix(Prec),
    Infix(Prec, Prec)
}
impl HasPrec for Fixity {
    fn prec(&self, side: Side) -> Prec {
        match (*self, side) {
            (Nilfix,          _)     => NO_PREC,
            (Prefix(_),       Left)  => NO_PREC,
            (Prefix(prec),    Right) => prec,
            (Postfix(prec),   Left)  => prec,
            (Postfix(_),      Right) => NO_PREC,
            (Infix(lprec, _), Left)  => lprec,
            (Infix(_, rprec), Right) => rprec
        }
    }
}
