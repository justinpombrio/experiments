use crate::ast::{Expr, Id, Loc, Located};
use crate::memory::{MemoryError, Value};
use crate::show_error::ShowError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Bug in TC! Expected {expected} but found {actual}.")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
        loc: Loc,
    },
    #[error("Bug in TC! Expected {expected} args, but got {actual}.")]
    WrongNumArgs {
        expected: usize,
        actual: usize,
        loc: Loc,
    },
    #[error("Bug in TC! Variable '{}' not found.", .0.inner)]
    UnboundId(Located<Id>),
    #[error("{error}")]
    MemoryError { error: MemoryError, loc: Loc },
    #[error("Bug in Comptime! Encountered leftover comptime code at runtime.")]
    LeftoverComptime(Loc),
}

impl RuntimeError {
    fn type_mismatch(
        expected: &'static str,
        actual: &'static str,
        expr: &Located<Expr>,
    ) -> RuntimeError {
        RuntimeError::TypeMismatch {
            expected,
            actual,
            loc: expr.loc,
        }
    }
}

impl Value {
    pub fn unwrap_int(self, expr: &Located<Expr>) -> Result<i32, RuntimeError> {
        if let Value::Int(n) = self {
            Ok(n)
        } else {
            Err(RuntimeError::type_mismatch("Int", self.type_name(), expr))
        }
    }
}

impl ShowError for RuntimeError {
    fn kind(&self) -> &'static str {
        use RuntimeError::*;

        match self {
            TypeMismatch { .. } | WrongNumArgs { .. } | UnboundId(_) => "bug in type checker",
            MemoryError { .. } => "memory error",
            LeftoverComptime(_) => "leftover comptime code",
        }
    }

    fn loc(&self) -> Option<Loc> {
        use RuntimeError::*;

        match self {
            TypeMismatch { loc, .. } => Some(*loc),
            WrongNumArgs { loc, .. } => Some(*loc),
            MemoryError { loc, .. } => Some(*loc),
            UnboundId(id) => Some(id.loc),
            LeftoverComptime(loc) => Some(*loc),
        }
    }

    fn short_message(&self) -> String {
        use RuntimeError::*;

        match self {
            TypeMismatch {
                expected, actual, ..
            } => format!("expected '{expected}', found '{actual}'"),
            WrongNumArgs { expected, .. } => format!("expected {expected} args"),
            MemoryError { error, .. } => error.to_string(),
            UnboundId(id) => format!("var {} not found", id.inner),
            LeftoverComptime(_) => "leftover comptime code".to_owned(),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
