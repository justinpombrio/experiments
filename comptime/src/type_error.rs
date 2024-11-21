use crate::ast::{Id, Loc, Located, Type};
use crate::show_error::ShowError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Variable {} not found", .0.inner)]
    UnboundId(Located<Id>),

    #[error("Function {} not found", .0.inner)]
    UnboundFunc(Located<Id>),

    #[error("Expected type {expected} but found {actual}")]
    TypeMismatch {
        expected: Type,
        actual: Type,
        loc: Loc,
    },

    #[error("Expected function type but found {actual}")]
    ExpectedFunction { actual: Type, loc: Loc },

    #[error("Expected {expected} args, but got {actual}.")]
    WrongNumArgs {
        expected: usize,
        actual: usize,
        loc: Loc,
    },

    #[error("Already in #comptime.")]
    NestedComptime(Loc),
}

impl ShowError for TypeError {
    fn kind(&self) -> &'static str {
        "type error"
    }

    fn loc(&self) -> Option<Loc> {
        use TypeError::*;

        match self {
            UnboundId(id) | UnboundFunc(id) => Some(id.loc),
            WrongNumArgs { loc, .. } => Some(*loc),
            TypeMismatch { loc, .. } => Some(*loc),
            ExpectedFunction { loc, .. } => Some(*loc),
            NestedComptime(loc) => Some(*loc),
        }
    }

    fn short_message(&self) -> String {
        use TypeError::*;

        match self {
            UnboundId { .. } => "variable not found".to_owned(),
            UnboundFunc { .. } => "function not found".to_owned(),
            TypeMismatch { expected, .. } => format!("expected {expected}"),
            ExpectedFunction { .. } => format!("expected function"),
            WrongNumArgs { expected, .. } => format!("expected {expected} arguments"),
            NestedComptime(_) => format!("nested #comptime"),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
