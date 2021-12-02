use std::fmt;
use std::iter;

use util::display_sep;
use source::Span;
use item::Lexeme;
use grammar::Operator;

use self::Term::*;
use self::Context::*;



pub enum Term<'s, 'g> {
    EmptyTerm(Span<'s>),
    LexTerm(Lexeme<'s, 'g>),
    JuxtTerm(Box<Term<'s, 'g>>, Box<Term<'s, 'g>>),
    StxTerm(&'g Operator, Vec<Term<'s, 'g>>),
    MacTerm(&'g Operator, Vec<Term<'s, 'g>>)
}
impl<'s, 'g> Term<'s, 'g> {
    pub fn juxt(left: Term<'s, 'g>, right: Term<'s, 'g>) -> Term<'s, 'g> {
        JuxtTerm(Box::new(left), Box::new(right))
    }
}
impl<'s, 'g> fmt::Display for Term<'s, 'g> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EmptyTerm(_) => write!(f, "."),
            &LexTerm(ref lex) => {
                // TODO: proper escaping
                let s = format!("{}", lex);
                let mut word = s.as_str();
                if word == "[" { word = "\\[" }
                if word == "]" { word = "\\]" }
                if word == "(" { word = "\\(" }
                if word == ")" { word = "\\)" }
                write!(f, "{}", word)
            },
            &JuxtTerm(ref left, ref right) => {
                write!(f, "[Juxt {} {}]", *left, *right)
            },
            &StxTerm(ref op, ref body) => {
                try!(write!(f, "[{} ", op));
                try!(display_sep(f, " ", body.iter()));
                write!(f, "]")
            },
            &MacTerm(ref op, ref body) => {
                try!(write!(f, "({} ", op));
                try!(display_sep(f, " ", body.iter()));
                write!(f, ")")
            }
        }
    }
}

