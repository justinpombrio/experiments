mod bytecode;
mod compiler;
mod expr;
mod sexpr;
mod source;

use bytecode::{Instr, Value};
use compiler::{Compiler, Registry, TypeError};
use expr::{Compiled, Expr, Type};
use source::Src;
use Type::Int;

macro_rules! type_err {
    ($loc:expr, $fmt_str:expr $( , $args:expr )*) => {
        return Err(TypeError)
    };
}

fn compile_int<'s>(_comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let n: i32 = match src.as_str().parse::<i32>() {
        Ok(n) => n,
        Err(_) => type_err!(loc, "bad int"),
    };
    Ok(Compiled::Expr(Expr {
        loc: src.loc(),
        typ: Type::Int,
        code: vec![Instr::Push(Value::Int(n))],
    }))
}

/// $x:Expr + $y:Expr
fn compile_add<'s>(comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let loc = src.loc();
    let x = comp.compile(src.args()[0])?.into_expr();
    let mut y = comp.compile(src.args()[1])?.into_expr();

    let typ = match (x.typ, y.typ) {
        (Int, Int) => Int,
        (_, _) => type_err!(args[0], "Adding non ints", x.typ, y.typ),
    };

    let mut code = x.code;
    code.append(&mut y.code);
    code.push(Instr::Add);

    Ok(Compiled::Expr(Expr { loc, typ, code }))
}

fn std_registry() -> Registry {
    let mut registry = Registry::new();
    registry.add_fragment("int", compile_int);
    registry.add_fragment("+", compile_add);
    registry
}

fn main() {
    use sexpr::parse_sexpr;
    use typed_arena::Arena;

    let arena = Arena::new();
    let source = "(+ (+ 1 2) 3)";
    let add_expr = parse_sexpr(&arena, source).unwrap();

    let registry = std_registry();
    let mut compiler = Compiler::new(&registry);
    let expr = compiler.compile(add_expr).unwrap().into_expr();
    println!("{:?}", expr.code);
    println!("ok");
}
