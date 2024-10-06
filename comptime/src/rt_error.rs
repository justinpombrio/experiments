use crate::value::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RtError {
    #[error("Bug in TC! Expected {expected} but found {found} during {operation}.")]
    TypeCheckingBug {
        expected: &'static str,
        found: &'static str,
        operation: &'static str,
    },
}

impl RtError {
    pub fn err_tc(expected: &'static str, found: &Value, operation: &'static str) -> RtError {
        RtError::TypeCheckingBug {
            expected,
            found: found.typename(),
            operation,
        }
    }
}
