use crate::ast::{Expr, Func, FuncType, Id, Prog, Type};
use parser_ll1::{choice, tuple, CompiledParser, Grammar, GrammarError, Parser, Recursive};
use std::str::FromStr;

// fn add1(n: Int)->Int { n + 1 } fn main()->Int { call add1(2) }

fn id_parser(g: &mut Grammar) -> Result<impl Parser<Id> + Clone, GrammarError> {
    Ok(g.regex("variable", "[a-zA-Z_][a-zA-Z0-9]*")?
        .span(|span| span.substr.to_owned()))
}

fn parenthesized_list<T>(
    g: &mut Grammar,
    name: &'static str,
    parser: impl Parser<T> + Clone,
) -> Result<impl Parser<Vec<T>> + Clone, GrammarError>
where
    T: Clone,
{
    let elems_p = parser.many_sep0(g.string(",")?);
    let none_p = g.string("()")?.constant(Vec::new());
    let some_p = tuple(name, (g.string("(")?, elems_p, g.string(")")?)).map(|(_, elems, _)| elems);
    Ok(choice(name, (none_p, some_p)))
}

fn expr_parser(g: &mut Grammar) -> Result<impl Parser<Expr> + Clone, GrammarError> {
    let id_p = id_parser(g)?;
    let expr_p = Recursive::new("expression");

    let unit_p = g.string("()")?.constant(Expr::Unit);
    let int_p = g
        .regex("int", "[1-9][0-9]*")?
        .try_span(|s| i32::from_str(s.substr))
        .map(Expr::Int);
    let id_expr_p = id_p.clone().map(Expr::Id);
    let paren_p = tuple(
        "parenthetical expression",
        (g.string("(")?, expr_p.refn(), g.string(")")?),
    )
    .map(|(_, expr, _)| expr);

    // Id(Expr, ...)
    let args_p = parenthesized_list(g, "function arguments", expr_p.refn())?;
    let call_p = tuple("function call", (g.string("call")?, id_p, args_p))
        .map(|(_, func, args)| Expr::Call(func, args));

    let atom_p = choice("expression", (unit_p, int_p, id_expr_p, paren_p, call_p));

    let add_p = atom_p.clone().fold_many1(
        tuple("addition expression", (g.string("+")?, atom_p)),
        |x, (_, y)| Expr::Add(Box::new(x), Box::new(y)),
    );

    Ok(expr_p.define(add_p))
}

fn type_parser(g: &mut Grammar) -> Result<impl Parser<Type> + Clone, GrammarError> {
    let type_p = Recursive::new("type");
    let unit_p = g.string("()")?.constant(Type::Unit);
    let int_p = g.string("Int")?.constant(Type::Int);

    // fn (Type, ...) -> Type
    let params_p = parenthesized_list(g, "function parameters", type_p.refn())?;
    let func_p = tuple(
        "function type",
        (g.string("fn")?, params_p, g.string("->")?, type_p.refn()),
    )
    .map(|(_, params, _, returns)| {
        Type::Func(FuncType {
            params,
            returns: Box::new(returns),
        })
    });

    Ok(type_p.define(choice("type", (unit_p, int_p, func_p))))
}

fn prog_parser(g: &mut Grammar) -> Result<impl Parser<Prog> + Clone, GrammarError> {
    let id_p = id_parser(g)?;
    let expr_p = expr_parser(g)?;
    let type_p = type_parser(g)?;

    // fn Id(Id: Type, ...) -> Type { Expr }
    let param_p = tuple(
        "function parameter",
        (id_p.clone(), g.string(":")?, type_p.clone()),
    )
    .map(|(param, _, ty)| (param, ty));
    let params_p = parenthesized_list(g, "function parameters", param_p)?;
    let func_p = tuple(
        "function",
        (
            g.string("fn")?,
            id_p,
            params_p,
            g.string("->")?,
            type_p,
            g.string("{")?,
            expr_p,
            g.string("}")?,
        ),
    )
    .map(|(_, name, params, _, returns, _, body, _)| Func {
        name,
        params,
        returns,
        body,
    });

    let prog_p = func_p.many0().map(|funcs| Prog { funcs });
    Ok(prog_p)
}

pub fn make_prog_parser() -> Result<impl CompiledParser<Prog>, GrammarError> {
    let mut g = Grammar::with_whitespace("[ \t\r\n]+")?;
    let prog_p = prog_parser(&mut g)?;
    g.compile_parser(prog_p)
}
