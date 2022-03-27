mod bytecode;
mod compiler;
mod expr;
mod sexpr;
mod source;

use bytecode::{Code, Instr, Value};
use compiler::{Compiler, Registry, TypeError};
use expr::{Compiled, Expr, Type};
use source::Src;

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
        code: Code(vec![Instr::Push(Value::Int(n))]),
    }))
}

/// $x:Expr + $y:Expr
fn compile_add<'s>(comp: &mut Compiler<'s, '_>, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let loc = src.loc();
    let x = comp.compile("Expr", src.args()[0])?.into_expr();
    let mut y = comp.compile("Expr", src.args()[1])?.into_expr();

    let typ = match (x.typ, y.typ) {
        (Type::Int, Type::Int) => Type::Int,
        // (_, _) => type_err!(loc, "Adding non ints {} {}", x.typ, y.typ),
    };

    let mut code = x.code;
    code.0.append(&mut y.code.0);
    code.0.push(Instr::Add);

    Ok(Compiled::Expr(Expr { loc, typ, code }))
}

/// let $v:id = $x:Expr in $b:Expr
fn compile_let<'s>(comp: &mut Compiler<'s, '_>, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let loc = src.loc();
    let expr = comp.compile("Expr", src.args()[1])?.into_expr();
    let name = comp.compile("Pattern", src.args()[0])?.into_id();
    let reg = comp.push_var(name, expr.typ);
    let mut body = comp.compile("Expr", src.args()[2])?.into_expr();
    comp.pop_var(name, reg);

    let mut code = expr.code;
    code.0.push(Instr::Push(Value::Reg(reg)));
    code.0.push(Instr::SetReg);
    code.0.append(&mut body.code.0);

    Ok(Compiled::Expr(Expr {
        loc,
        typ: body.typ,
        code,
    }))
}

/// $v:Id
fn compile_id<'s>(comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let var = src.loc.source;
    let (typ, reg) = if let Some((typ, reg)) = comp.lookup_var(var) {
        (typ, reg)
    } else {
        type_err!(src.loc(), "Unbound variable {}", var)
    };
    Ok(Compiled::Expr(Expr {
        loc: src.loc(),
        typ,
        code: Code(vec![Instr::Push(Value::Reg(reg)), Instr::GetReg]),
    }))
}

/// $v:Id in pattern position
fn compile_pattern_id<'s>(_comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    Ok(Compiled::Id(src.loc.source))
}

fn std_registry() -> Registry {
    let mut registry = Registry::new();

    // Exprs
    registry.add_fragment("Expr", "int", compile_int);
    registry.add_fragment("Expr", "+", compile_add);
    registry.add_fragment("Expr", "id", compile_id);
    registry.add_fragment("Expr", "let", compile_let);

    // Patterns
    registry.add_fragment("Pattern", "id", compile_pattern_id);

    registry
}

fn main() {
    use sexpr::parse_sexpr;
    use typed_arena::Arena;

    let registry = std_registry();
    let mut compiler = Compiler::new(&registry);

    let arena = Arena::new();
    let source = "(+ (+ 1 2) 3)";
    let expr = parse_sexpr(&arena, source).unwrap();
    let expr = compiler.compile("Expr", expr).unwrap().into_expr();
    println!("{}", expr.code);

    let arena = Arena::new();
    let source = "(let x 1 (+ x 2))";
    let expr = parse_sexpr(&arena, source).unwrap();
    let expr = compiler.compile("Expr", expr).unwrap().into_expr();
    println!("{}", expr.code);

    println!("ok");
}
