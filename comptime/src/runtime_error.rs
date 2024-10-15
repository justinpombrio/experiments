use crate::ast::{Id, Value};
use crate::pretty_error::PrettyError;
use parser_ll1::Position;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Bug in TC! Expected {expected} but found {actual} during {operation}.")]
    TypeCheckingBug {
        expected: &'static str,
        actual: &'static str,
        operation: &'static str,
    },
    #[error("Bug in TC! Wrong number of arguments to function {func}. Expected {expected}, found {actual}.")]
    WrongNumArgs {
        func: Id,
        expected: usize,
        actual: usize,
    },
    #[error("Bug in TC! Var '{id}' not found.")]
    ScopeBug { id: Id },
}

impl RuntimeError {
    pub fn err_tc(expected: &'static str, actual: &Value, operation: &'static str) -> RuntimeError {
        RuntimeError::TypeCheckingBug {
            expected,
            actual: actual.type_name(),
            operation,
        }
    }

    pub fn err_id(id: Id) -> RuntimeError {
        RuntimeError::ScopeBug { id }
    }
}

impl Value {
    pub fn unwrap_int(self, context: &'static str) -> Result<i32, RuntimeError> {
        if let Value::Int(n) = self {
            Ok(n)
        } else {
            Err(RuntimeError::err_tc("Int", &self, context))
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

    fn src_loc(&self) -> Option<(Position, Position)> {
        None
    }

    fn short_message(&self) -> String {
        use RuntimeError::*;

        match self {
            TypeCheckingBug {
                expected, actual, ..
            } => format!("expected '{expected}', found '{actual}'"),
            WrongNumArgs { expected, .. } => format!("expected {expected} args"),
            ScopeBug { id } => format!("var {id} not found"),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
