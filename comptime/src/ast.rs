use std::fmt;

pub type Var = String;

#[derive(Clone, Debug)]
pub enum Value {
    Unit,
    Int(i32),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Unit,
    Int(i32),
    Add(Box<Expr>, Box<Expr>),
    Id(Var),
    Call(Var, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub struct Prog {
    pub funcs: Vec<Func>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Unit,
    Int,
    Func {
        params: Vec<Type>,
        returns: Box<Type>,
    },
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: Var,
    pub params: Vec<(Var, Type)>,
    pub returns: Type,
    pub body: Expr,
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
