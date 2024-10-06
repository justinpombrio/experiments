use crate::expr::Expr;
use parser_ll1::{CompiledParser, Grammar, GrammarError, Parser, Recursive};
use std::str::FromStr;

pub fn make_expr_parser() -> Result<impl CompiledParser<Expr>, GrammarError> {
    let mut g = Grammar::with_whitespace("[ \t\r\n]+")?;

    let expr_p = Recursive::new("expression");

    let num_p = g
        .regex("number", "[1-9][0-9]*")?
        .try_span(|s| i32::from_str(s.substr))
        .map(Expr::Num);

    let add_p = num_p
        .clone()
        .many_sep1(g.string("+")?)
        .map(|nums| Expr::Add(nums));

    let expr_p = expr_p.define(add_p);

    g.compile_parser(expr_p)
}
