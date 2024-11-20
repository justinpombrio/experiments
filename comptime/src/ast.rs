use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}
pub type Loc = (Pos, Pos);

pub type Id = String;

#[derive(Debug, Clone)]
pub struct Located<T> {
    pub loc: Loc,
    pub inner: T,
}

#[derive(Debug, Clone)]
pub struct Prog {
    pub funcs: Vec<Located<Func>>,
    pub main: Located<Expr>,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: Located<Id>,
    pub params: Vec<Param>,
    pub returns: Type,
    pub body: Located<Expr>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub id: Id,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Unit,
    Int(i32),
    Id(Located<Id>),
    Sum(Vec<Located<Expr>>),
    Let(Located<Id>, Box<Located<Expr>>, Box<Located<Expr>>),
    Call(Located<Id>, Vec<Located<Expr>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Int,
    Func(FuncType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<Type>,
    pub returns: Box<Type>,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Int => write!(f, "Int"),
            Type::Func(func_type) => write!(f, "{}", func_type),
        }
    }
}

impl fmt::Display for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn(")?;
        if let Some(first_ty) = self.params.first() {
            write!(f, "{}", first_ty)?;
        }
        for ty in self.params.iter().skip(1) {
            write!(f, ", {}", ty)?;
        }
        write!(f, ") -> ")?;
        write!(f, "{}", self.returns)
    }
}
