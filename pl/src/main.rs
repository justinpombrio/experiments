use std::collections::HashMap;
use std::convert::TryInto;

macro_rules! type_err {
    ($loc:expr, $fmt_str:expr $( , $args:expr )*) => {
        return Err(TypeError)
    };
}

#[derive(Debug, Clone, Copy)]
struct Srcloc<'s>(&'s str);

#[derive(Debug, Clone, Copy)]
struct Src<'s> {
    loc: Srcloc<'s>,
    construct: &'s str,
    args: &'s [Src<'s>],
}

#[derive(Debug)]
struct TypeError;

impl<'s> Srcloc<'s> {
    fn as_str(&self) -> &str {
        self.0
    }
}

impl<'s> Src<'s> {
    fn as_str(&self) -> &str {
        self.loc.as_str()
    }

    fn construct(&self) -> &str {
        self.construct
    }

    fn args(&self) -> &[Src] {
        self.args
    }
}

#[derive(Debug)]
enum Compiled<'s> {
    Type,
    Expr(Expr<'s>),
}

impl<'s> Compiled<'s> {
    fn into_expr(self) -> Result<Expr<'s>, TypeError> {
        match self {
            // TODO: srcloc?
            Compiled::Expr(expr) => Ok(expr),
            _ => type_err!(0, "not an expr"),
        }
    }
}

#[derive(Debug)]
enum Type {
    Int,
    Float,
}
use Type::*;

#[derive(Debug)]
struct Expr<'s> {
    loc: Srcloc<'s>,
    typ: Type,
    code: Code,
}

type Code = Vec<Instr>;
#[derive(Debug)]
enum Instr {
    Int(i32),
    Float(f32),
    Add,
}

type Fragment = for<'s> fn(&mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError>;

struct Registry {
    fragments: HashMap<String, Fragment>,
}

impl Registry {
    fn new() -> Registry {
        Registry {
            fragments: HashMap::new(),
        }
    }

    fn add_fragment(&mut self, con: &str, fragment: Fragment) {
        self.fragments.insert(con.to_owned(), fragment);
    }

    fn get_fragment<'s>(&self, con: &str) -> Option<Fragment> {
        self.fragments.get(con).copied()
    }
}

struct Compiler<'r> {
    registry: &'r Registry,
}

impl<'r> Compiler<'r> {
    fn new(registry: &'r Registry) -> Compiler<'r> {
        Compiler { registry }
    }

    fn compile<'s>(&mut self, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
        let fragment = match self.registry.get_fragment(src.construct()) {
            Some(fragment) => fragment,
            None => type_err!(src.loc(), "missing fragment"),
        };
        let mut args = vec![];
        for arg in src.args {
            args.push(self.compile(*arg)?);
        }
        fragment(self, src)
    }
}

fn compile_int<'s>(comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let n: i32 = match src.loc.as_str().parse::<i32>() {
        Ok(n) => n,
        Err(_) => type_err!(loc, "bad int"),
    };
    Ok(Compiled::Expr(Expr {
        loc: src.loc,
        typ: Type::Int,
        code: vec![Instr::Int(n)],
    }))
}

/// $x:Expr + $y:Expr
fn compile_add<'s>(comp: &mut Compiler, src: Src<'s>) -> Result<Compiled<'s>, TypeError> {
    let loc = src.loc;
    let x = comp.compile(src.args[0])?.into_expr()?;
    let mut y = comp.compile(src.args[1])?.into_expr()?;

    let typ = match (x.typ, y.typ) {
        (Int, Int) => Int,
        (Float, Float) => Float,
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
    registry.add_fragment("add", compile_add);
    registry
}

fn main() {
    let one_expr = Src {
        construct: "int",
        loc: Srcloc("1"),
        args: &[],
    };
    let two_expr = Src {
        construct: "int",
        loc: Srcloc("2"),
        args: &[],
    };
    let add_expr = Src {
        construct: "add",
        loc: Srcloc("1 + 2"),
        args: &[one_expr, two_expr],
    };

    let registry = std_registry();
    let mut compiler = Compiler::new(&registry);
    let expr = compiler.compile(add_expr).unwrap().into_expr().unwrap();
    println!("{:?}", expr.code);
    println!("ok");
}
