#![allow(clippy::precedence)]

use crate::ast::{Expr, Func, FuncType, Id, Located, Param, Prog, Type};
use ppp::doc_examples::tree::{Tree, TreeCondition, TreeNotation, TreeStyleLabel};
use ppp::doc_examples::BasicStyle;
use ppp::notation_constructors::{
    child, count, empty, flat, fold, left, lit, nl, right, style, text, Count, Fold,
};
use ppp::{Line, Notation};
use std::fmt;
use std::sync::LazyLock;

// green, magenta, blue, yellow
const CONSTANT_STYLE: &str = "magenta";
const KEYWORD_STYLE: &str = "yellow";
// const NUMBER_STYLE: &str = "blue";
// const COMMENT_STYLE: &str = "yellow";

pub fn pretty_print(prog: &Prog, width: u16) -> String {
    use ppp::FocusTarget;

    let mut lines = Vec::new();
    let tree = prog.show();
    let (_prev_lines, focused_line, next_lines) =
        ppp::pretty_print(&tree, width, &[], FocusTarget::Start, None).unwrap();
    lines.push(Line::from(focused_line));
    for line in next_lines {
        lines.push(line.unwrap());
    }
    lines_to_string(lines).unwrap()
}

fn comma_sep() -> Notation<TreeStyleLabel, TreeCondition> {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(", ") + flat(right()),
    });
    let multi_seq = fold(Fold {
        first: child(0),
        join: left() + lit(",") ^ right(),
    });
    count(Count {
        zero: empty(),
        one: child(0),
        many: single_seq | multi_seq,
    })
}

fn infix_sep(sep: &'static str) -> Notation<TreeStyleLabel, TreeCondition> {
    let single_seq = fold(Fold {
        first: flat(child(0)),
        join: left() + lit(" ") + lit(sep) + lit(" ") + flat(right()),
    });
    let multi_seq = fold(Fold {
        first: child(0),
        join: left() ^ lit(sep) + lit(" ") + right(),
    });
    count(Count {
        zero: empty(),
        one: child(0),
        many: single_seq | multi_seq,
    })
}

static TYPE_UNIT_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| style(CONSTANT_STYLE, lit("()")).validate().unwrap());

static TYPE_INT_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| style(CONSTANT_STYLE, lit("Int")).validate().unwrap());

static TYPE_FUNC_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| {
    let prefix = style(KEYWORD_STYLE, lit("fn")) + lit("(");
    let suffix = lit(") -> ") + child(1);

    let single = prefix.clone() + child(0) + suffix.clone();
    let multi = prefix + (4 >> child(0)) + nl() + suffix;
    let options = single | multi;

    options.validate().unwrap()
});

static TYPE_PARAMS_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| comma_sep().validate().unwrap());

static EXPR_UNIT_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| style(CONSTANT_STYLE, lit("()")).validate().unwrap());

static EXPR_INT_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| style(CONSTANT_STYLE, text()).validate().unwrap());

static ID_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| text().validate().unwrap());

static EXPR_SUM_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| infix_sep("+").validate().unwrap());

static FUNC_NAME_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| text().validate().unwrap());

static EXPR_ARGS_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| comma_sep().validate().unwrap());

static EXPR_CALL_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| {
    let single = text() + lit("(") + flat(child(0)) + lit(")");
    let multi = text() + lit("(") + (4 >> child(0)) + nl() + lit(")");
    let options = single | multi;

    options.validate().unwrap()
});

static PARAM_NOTATION: LazyLock<TreeNotation> =
    LazyLock::new(|| (child(0) + lit(": ") + child(1)).validate().unwrap());

static PARAMS_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| comma_sep().validate().unwrap());

static FUNC_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| {
    let prefix = style(KEYWORD_STYLE, lit("fn")) + lit(" ") + child(0) + lit("(");
    let suffix = lit(") -> ") + child(2) + lit(" {") + (4 >> child(3)) ^ lit("}");

    let single = prefix.clone() + child(1) + suffix.clone();
    let multi = prefix + (4 >> child(1)) + nl() + suffix;
    let options = single | multi;

    options.validate().unwrap()
});

static PROG_NOTATION: LazyLock<TreeNotation> = LazyLock::new(|| {
    fold(Fold {
        first: child(0),
        join: left() ^ empty() ^ right(),
    })
    .validate()
    .unwrap()
});

trait Show {
    fn show(&self) -> Tree<BasicStyle>;
}

fn leaf(notation: &'static TreeNotation) -> Tree<BasicStyle> {
    Tree::new_branch(notation, Vec::new())
}

fn leaf_text(notation: &'static TreeNotation, text: String) -> Tree<BasicStyle> {
    Tree::new_text(notation, text)
}

fn branch<const N: usize>(
    notation: &'static TreeNotation,
    children: [Tree<BasicStyle>; N],
) -> Tree<BasicStyle> {
    Tree::new_branch(notation, children.into_iter().collect::<Vec<_>>())
}

fn branch_seq<'a, T: Show + 'a>(
    notation: &'static TreeNotation,
    children: impl IntoIterator<Item = &'a T>,
) -> Tree<BasicStyle> {
    Tree::new_branch(
        notation,
        children.into_iter().map(|p| p.show()).collect::<Vec<_>>(),
    )
}

impl<T: Show> Show for Located<T> {
    fn show(&self) -> Tree<BasicStyle> {
        self.inner.show()
    }
}

impl Show for Id {
    fn show(&self) -> Tree<BasicStyle> {
        leaf_text(&ID_NOTATION, self.to_owned())
    }
}

impl Show for Type {
    fn show(&self) -> Tree<BasicStyle> {
        use Type::*;

        match self {
            Unit => leaf(&TYPE_UNIT_NOTATION),
            Int => leaf(&TYPE_INT_NOTATION),
            Func(func_ty) => func_ty.show(),
        }
    }
}

impl Show for FuncType {
    fn show(&self) -> Tree<BasicStyle> {
        branch(
            &TYPE_FUNC_NOTATION,
            [
                branch_seq(&TYPE_PARAMS_NOTATION, &self.params),
                self.returns.show(),
            ],
        )
    }
}

impl Show for Expr {
    fn show(&self) -> Tree<BasicStyle> {
        use Expr::*;

        match self {
            Unit => leaf(&EXPR_UNIT_NOTATION),
            Int(i) => leaf_text(&EXPR_INT_NOTATION, i.to_string()),
            Id(id) => id.show(),
            Sum(terms) => branch_seq(&EXPR_SUM_NOTATION, terms),
            Call(func, args) => branch(
                &EXPR_CALL_NOTATION,
                [
                    leaf_text(&FUNC_NAME_NOTATION, func.inner.clone()),
                    branch_seq(&EXPR_ARGS_NOTATION, args),
                ],
            ),
        }
    }
}

impl Show for Param {
    fn show(&self) -> Tree<BasicStyle> {
        branch(&PARAM_NOTATION, [self.id.show(), self.ty.show()])
    }
}

impl Show for Func {
    fn show(&self) -> Tree<BasicStyle> {
        branch(
            &FUNC_NOTATION,
            [
                self.name.show(),
                branch_seq(&PARAMS_NOTATION, &self.params),
                self.returns.show(),
                self.body.show(),
            ],
        )
    }
}

impl Show for Prog {
    fn show(&self) -> Tree<BasicStyle> {
        branch_seq(&PROG_NOTATION, &self.funcs)
    }
}

fn lines_to_string<'a>(lines: Vec<Line<'a, &'a Tree<BasicStyle>>>) -> Result<String, fmt::Error> {
    use colored::Colorize;
    use ppp::doc_examples::Color::*;
    use std::fmt::Write;

    let mut string = String::new();
    let w = &mut string;
    for line in lines {
        for segment in line.segments {
            let str = segment.str;
            match segment.style.color {
                White => write!(w, "{}", str.white())?,
                Black => write!(w, "{}", str.black())?,
                Red => write!(w, "{}", str.red())?,
                Green => write!(w, "{}", str.green())?,
                Yellow => write!(w, "{}", str.yellow())?,
                Blue => write!(w, "{}", str.blue())?,
                Magenta => write!(w, "{}", str.magenta())?,
                Cyan => write!(w, "{}", str.cyan())?,
            };
        }
        writeln!(w)?;
    }
    Ok(string)
}
