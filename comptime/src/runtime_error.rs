use crate::ast::{Expr, Id, Loc, Located, Type, Value};
use crate::pretty_error::PrettyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Bug in TC! Expected {expected} but found {actual}.")]
    TypeMismatch {
        expected: Type,
        actual: Type,
        loc: Loc,
    },
    #[error("Bug in TC! Wrong number of arguments to function {}. Expected {expected}, found {actual}.", defsite.inner)]
    WrongNumArgs {
        callsite: Located<Id>,
        defsite: Located<Id>,
        expected: usize,
        actual: usize,
    },
    #[error("Bug in TC! Var '{}' not found.", .0.inner)]
    UnboundId(Located<Id>),
}

fn type_mismatch(expected: Type, actual: Type, expr: &Located<Expr>) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected,
        actual,
        loc: expr.loc,
    }
}

impl Value {
    pub fn unwrap_int(self, expr: &Located<Expr>) -> Result<i32, RuntimeError> {
        if let Value::Int(n) = self {
            Ok(n)
        } else {
            Err(type_mismatch(Type::Int, self.type_of(), expr))
        }
    }
}

impl PrettyError for RuntimeError {
    fn kind(&self) -> &'static str {
        use RuntimeError::*;

        match self {
            TypeMismatch { .. } | WrongNumArgs { .. } | UnboundId(_) => "bug in type checker",
        }
    }

    fn loc(&self) -> Option<Loc> {
        use RuntimeError::*;

        match self {
            TypeMismatch { loc, .. } => Some(*loc),
            WrongNumArgs { callsite, .. } => Some(callsite.loc),
            UnboundId(id) => Some(id.loc),
        }
    }

    fn short_message(&self) -> String {
        use RuntimeError::*;

        match self {
            TypeMismatch {
                expected, actual, ..
            } => format!("expected '{expected}', found '{actual}'"),
            WrongNumArgs { expected, .. } => format!("expected {expected} args"),
            UnboundId(id) => format!("var {} not found", id.inner),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
