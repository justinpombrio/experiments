use super::bytecode::Code;
use super::source::Srcloc;

#[derive(Debug)]
pub enum Compiled<'s> {
    Id(&'s str),
    Expr(Expr<'s>),
}

impl<'s> Compiled<'s> {
    pub fn into_id(self) -> &'s str {
        match self {
            Compiled::Id(s) => s,
            _ => panic!("not an id"),
        }
    }

    pub fn into_expr(self) -> Expr<'s> {
        match self {
            Compiled::Expr(expr) => expr,
            _ => panic!("not an expr"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
}

#[derive(Debug)]
pub struct Expr<'s> {
    pub loc: Srcloc<'s>,
    pub typ: Type,
    pub code: Code,
}
