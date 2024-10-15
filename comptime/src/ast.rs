use parser_ll1::Position;
use std::fmt;

pub type Loc = (Position, Position);

pub type Id = String;

#[derive(Clone, Debug)]
pub enum Value {
    Unit,
    Int(i32),
}

#[derive(Clone, Debug)]
pub struct Located<T> {
    pub loc: Loc,
    pub inner: T,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Unit,
    Int(i32),
    Id(Located<Id>),
    Add(Box<Located<Expr>>, Box<Located<Expr>>),
    Call(Located<Id>, Vec<Located<Expr>>),
}

#[derive(Clone, Debug)]
pub struct Prog {
    pub funcs: Vec<Located<Func>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Int,
    Func(FuncType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuncType {
    pub params: Vec<Type>,
    pub returns: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: Id,
    pub params: Vec<(Id, Type)>,
    pub returns: Type,
    pub body: Located<Expr>,
}

pub fn end_loc(source: &str) -> Loc {
    let offset = source.len();
    let line = source.lines().count() - 1;
    let last_line = source.lines().last();
    let col = last_line.map(|l| l.len()).unwrap_or(0);
    let utf8_col = last_line.map(|l| l.chars().count()).unwrap_or(0);
    let pos = Position {
        offset,
        line: line as u32,
        col: col as u32,
        utf8_col: utf8_col as u32,
    };
    (pos, pos)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Int(n) => write!(f, "{}", n),
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "Unit",
            Value::Int(_) => "Int",
        }
    }
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
