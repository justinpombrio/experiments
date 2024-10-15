use crate::ast::{Expr, Id, Loc, Located, Type, Value};
use crate::pretty_error::PrettyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Bug in TC! Expected {expected} but found {actual}.")]
    TypeCheckingBug {
        expected: Type,
        actual: &'static str,
        loc: Loc,
    },
    #[error("Bug in TC! Wrong number of arguments to function {func_name}. Expected {expected}, found {actual}.")]
    WrongNumArgs {
        func_name: Id,
        expected: usize,
        actual: usize,
        loc: Loc,
    },
    #[error("Bug in TC! Var '{id}' not found.")]
    ScopeBug { id: Id, loc: Loc },
}

impl RuntimeError {
    pub fn err_tc(expected: Type, actual: &Value, expr: &Located<Expr>) -> RuntimeError {
        RuntimeError::TypeCheckingBug {
            expected,
            actual: actual.type_name(),
            loc: expr.loc,
        }
    }
}

impl Value {
    pub fn unwrap_int(self, expr: &Located<Expr>) -> Result<i32, RuntimeError> {
        if let Value::Int(n) = self {
            Ok(n)
        } else {
            Err(RuntimeError::err_tc(Type::Int, &self, expr))
        }
    }
}

impl PrettyError for RuntimeError {
    fn kind(&self) -> &'static str {
        use RuntimeError::*;

        match self {
            TypeCheckingBug { .. } | WrongNumArgs { .. } | ScopeBug { .. } => "bug in type checker",
        }
    }

    fn loc(&self) -> Option<Loc> {
        use RuntimeError::*;

        match self {
            TypeCheckingBug { loc, .. } => Some(*loc),
            WrongNumArgs { loc, .. } => Some(*loc),
            ScopeBug { loc, .. } => Some(*loc),
        }
    }

    fn short_message(&self) -> String {
        use RuntimeError::*;

        match self {
            TypeCheckingBug {
                expected, actual, ..
            } => format!("expected '{expected}', found '{actual}'"),
            WrongNumArgs { expected, .. } => format!("expected {expected} args"),
            ScopeBug { id, .. } => format!("var {id} not found"),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
