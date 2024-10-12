use crate::ast::{Value, Var};
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
        func: Var,
        expected: usize,
        found: usize,
    },
    #[error("Bug in SC! Var '{var}' not found.")]
    ScopeBug { var: Var },
}

impl RtError {
    pub fn err_tc(expected: &'static str, found: &Value, operation: &'static str) -> RtError {
        RtError::TypeCheckingBug {
            expected,
            found: found.type_name(),
            operation,
        }
    }

    pub fn err_var(var: Var) -> RtError {
        RtError::ScopeBug { var }
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
