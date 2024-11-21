use crate::ast::{Expr, Func, FuncType, Id, Located, Param, ParamMode, Pos, Prog, Type};
use parser_ll1::{choice, tuple, CompiledParser, Grammar, GrammarError, Parser, Recursive, Span};
use std::str::FromStr;

const VARIABLE_REGEX: &str = "[a-zA-Z_][a-zA-Z0-9_]*";
const VARIABLE_REGEX_COMPTIME: &str = "#[a-zA-Z_][a-zA-Z0-9_]*";

pub fn make_prog_parser() -> Result<impl CompiledParser<Prog>, GrammarError> {
    let mut g = Grammar::with_whitespace("[ \t\r\n]+")?;
    let prog_p = prog_parser(&mut g)?;
    g.compile_parser(prog_p)
}

fn located<T>(span: Span, inner: T) -> Located<T> {
    Located {
        loc: (
            Pos {
                line: span.start.line,
                col: span.start.utf8_col,
            },
            Pos {
                line: span.end.line,
                col: span.end.utf8_col,
            },
        ),
        inner,
    }
}

fn id_parser(g: &mut Grammar) -> Result<impl Parser<Located<Id>> + Clone, GrammarError> {
    Ok(g.regex("variable", VARIABLE_REGEX)?
        .span(|s| located(s, s.substr.to_owned())))
}

fn id_mode_parser(
    g: &mut Grammar,
) -> Result<impl Parser<(ParamMode, Located<Id>)> + Clone, GrammarError> {
    let rt_id = g
        .regex("variable", VARIABLE_REGEX)?
        .span(|s| (ParamMode::Runtime, located(s, s.substr.to_owned())));
    let ct_id = g
        .regex("variable", VARIABLE_REGEX_COMPTIME)?
        .span(|s| (ParamMode::Comptime, located(s, s.substr.to_owned())));
    Ok(choice("variable", (rt_id, ct_id)))
}

/// (P, ..., P)
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

fn expr_parser(g: &mut Grammar) -> Result<impl Parser<Located<Expr>> + Clone, GrammarError> {
    let id_mode_p = id_mode_parser(g)?;
    let id_p = id_parser(g)?;
    let expr_p = Recursive::<Located<Expr>>::new("expression");

    // ()
    let unit_p = g.string("()")?.span(|s| located(s, Expr::Unit));

    // <int>
    let int_p = g.regex("int", "[1-9][0-9]*")?.try_span(
        |s| -> Result<Located<Expr>, <i32 as FromStr>::Err> {
            Ok(located(s, Expr::Int(i32::from_str(s.substr)?)))
        },
    );

    // Id
    let id_expr_p = id_mode_p.clone().map(|(mode, id_loc)| Located {
        loc: id_loc.loc,
        inner: Expr::Id(mode, id_loc),
    });

    // (Expr)
    let paren_p = tuple(
        "parenthetical expression",
        (g.string("(")?, expr_p.refn(), g.string(")")?),
    )
    .map(|(_, expr, _)| expr);

    // ATOM ::= () | <int> | Id | (Expr)
    let atom_p = choice("expression", (unit_p, int_p, id_expr_p, paren_p));

    // (Expr, ...)
    let args_p = parenthesized_list(g, "function arguments", expr_p.refn())?;
    // Id(Expr, ...)
    let call_p = atom_p.and(args_p.opt()).map_span(|span, (atom, args)| {
        if let Some(args) = args {
            located(span, Expr::Call(Box::new(atom), args))
        } else {
            atom
        }
    });

    let add_p = call_p.many_sep1(g.string("+")?).map(|terms| {
        if terms.len() == 1 {
            terms.into_iter().next().unwrap()
        } else {
            Located {
                loc: (terms[0].loc.0, terms[terms.len() - 1].loc.1),
                inner: Expr::Sum(terms),
            }
        }
    });

    // let Id = Expr; Expr
    let let_runtime_p = g.string("let")?.constant(ParamMode::Runtime);
    let let_comptime_p = g.string("#let")?.constant(ParamMode::Comptime);
    let let_mode_p = choice("let expression", (let_runtime_p, let_comptime_p));
    let let_p = tuple(
        "let expression",
        (
            let_mode_p,
            id_p,
            g.string("=")?,
            expr_p.refn(),
            g.string(";")?,
            expr_p.refn(),
        ),
    )
    .map_span(|span, (mode, id, _, binding, _, body)| {
        located(span, Expr::Let(mode, id, Box::new(binding), Box::new(body)))
    });
    let stmt_p = choice("expression", (let_p, add_p));

    Ok(expr_p.define(stmt_p))
}

fn type_parser(g: &mut Grammar) -> Result<impl Parser<Type> + Clone, GrammarError> {
    let type_p = Recursive::<Type>::new("type");

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

    // Id: Type
    // #Id: Type
    let param_mode = g.string("#")?.opt().map(|opt| {
        if opt.is_some() {
            ParamMode::Comptime
        } else {
            ParamMode::Runtime
        }
    });
    let param_p = tuple(
        "function parameter",
        (param_mode, id_p.clone(), g.string(":")?, type_p.clone()),
    )
    .map_span(|span, (mode, param, _, ty)| {
        located(
            span,
            Param {
                id: param.inner,
                mode,
                ty,
            },
        )
    });

    // fn Id(Param, ...) -> Type { Expr }
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
            expr_p.clone(),
            g.string("}")?,
        ),
    )
    .map_span(|span, (_, name, params, _, returns, _, body, _)| {
        located(
            span,
            Func {
                name,
                params,
                returns,
                body,
            },
        )
    });

    let prog_p = func_p
        .many0()
        .and(expr_p)
        .map(|(funcs, main)| Prog { funcs, main });
    Ok(prog_p)
}
