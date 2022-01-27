use super::bytecode::Code;
use super::source::Srcloc;

#[derive(Debug)]
pub enum Compiled<'s> {
    Type,
    Expr(Expr<'s>),
}

impl<'s> Compiled<'s> {
    pub fn into_expr(self) -> Expr<'s> {
        match self {
            Compiled::Expr(expr) => expr,
            _ => panic!("not an expr"),
        }
    }
}

#[derive(Debug)]
pub enum Type {
    Int,
}

#[derive(Debug)]
pub struct Expr<'s> {
    pub loc: Srcloc<'s>,
    pub typ: Type,
    pub code: Code,
}
