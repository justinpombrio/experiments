use crate::ast::{Id, Value};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RtError {
    #[error("Bug in TC! Expected {expected} but found {found} during {operation}.")]
    TypeCheckingBug {
        expected: &'static str,
        found: &'static str,
        operation: &'static str,
    },
    #[error("Bug in TC! Wrong number of arguments to function {func}. Expected {expected}, found {found}.")]
    WrongNumArgs {
        func: Id,
        expected: usize,
        found: usize,
    },
    #[error("Bug in TC! Var '{id}' not found.")]
    ScopeBug { id: Id },
}

impl RtError {
    pub fn err_tc(expected: &'static str, found: &Value, operation: &'static str) -> RtError {
        RtError::TypeCheckingBug {
            expected,
            found: found.type_name(),
            operation,
        }
    }

    pub fn err_id(id: Id) -> RtError {
        RtError::ScopeBug { id }
    }
}

impl Value {
    pub fn unwrap_int(self, context: &'static str) -> Result<i32, RtError> {
        if let Value::Int(n) = self {
            Ok(n)
        } else {
            Err(RtError::err_tc("Int", &self, context))
        }
    }
}
