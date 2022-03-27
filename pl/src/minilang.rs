type Id = u32;

struct Type {}

enum Value {
    Int(i32),
    String(String),
}

enum Expr {
    Literal(Value),
    Var(Id),
    Let(Id, Type, Box<Expr>, Box<Expr>),
    Builtin(Op, Vec<Expr>),
}

enum Op {
    AddInt,
}
