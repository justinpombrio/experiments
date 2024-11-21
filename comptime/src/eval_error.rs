use crate::ast::{Id, Loc};
use crate::memory::MemoryError;
use crate::show_error::ShowError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvalErrorCase {
    #[error("Bug in TC! Expected {expected} but found {actual}.")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("Bug in TC! Expected {expected} args, but got {actual}.")]
    WrongNumArgs { expected: usize, actual: usize },
    #[error("Bug in TC! Variable '{0}' not found.")]
    UnboundId(Id),
    #[error("{0}")]
    MemoryError(MemoryError),
    #[error("Bug in Comptime! Encountered leftover comptime code at runtime.")]
    LeftoverComptime,
}

impl EvalErrorCase {
    fn kind(&self) -> &'static str {
        use EvalErrorCase::*;

        match self {
            TypeMismatch { .. } | WrongNumArgs { .. } | UnboundId(_) => "bug in type checker",
            MemoryError { .. } => "memory error",
            LeftoverComptime => "leftover comptime code",
        }
    }

    fn short_message(&self) -> String {
        use EvalErrorCase::*;

        match self {
            TypeMismatch {
                expected, actual, ..
            } => format!("expected '{expected}', found '{actual}'"),
            WrongNumArgs { expected, .. } => format!("expected {expected} args"),
            MemoryError(error) => error.to_string(),
            UnboundId(id) => format!("var {} not found", id),
            LeftoverComptime => "leftover comptime code".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    Runtime,
    Comptime,
}

#[derive(Error, Debug)]
#[error("{error}")]
pub struct EvalError {
    pub error: EvalErrorCase,
    pub loc: Loc,
    pub phase: Phase,
}

impl ShowError for EvalError {
    fn kind(&self) -> &'static str {
        self.error.kind()
    }

    fn loc(&self) -> Option<Loc> {
        Some(self.loc)
    }

    fn short_message(&self) -> String {
        self.error.short_message()
    }

    fn long_message(&self) -> String {
        format!("{}", &self.error)
    }
}
