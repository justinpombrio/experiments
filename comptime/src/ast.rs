use std::fmt;

pub type Id = String;

#[derive(Clone, Debug)]
pub enum Value {
    Unit,
    Int(i32),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Unit,
    Int(i32),
    Id(Id),
    Add(Box<Expr>, Box<Expr>),
    Call(Id, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub struct Prog {
    pub funcs: Vec<Func>,
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
