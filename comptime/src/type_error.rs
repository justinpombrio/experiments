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

    #[error("Wrong number of arguments passed to {}. Expected {expected} args, but received {actual}.", defsite.inner)]
    WrongNumArgs {
        callsite: Located<Id>,
        defsite: Located<Id>,
        expected: usize,
        actual: usize,
    },

    #[error("Missing main() function.")]
    MissingMain,

    #[error("The main() function's return type must be ().")]
    MainDoesNotReturnUnit,

    #[error("The main() function must not take any arguments.")]
    MainTakesArgs,
}

impl ShowError for TypeError {
    fn kind(&self) -> &'static str {
        "type error"
    }

    fn loc(&self) -> Option<Loc> {
        use TypeError::*;

        match self {
            UnboundId(id) | UnboundFunc(id) => Some(id.loc),
            WrongNumArgs { callsite, .. } => Some(callsite.loc),
            TypeMismatch { loc, .. } => Some(*loc),
            MissingMain | MainDoesNotReturnUnit | MainTakesArgs => None,
        }
    }

    fn short_message(&self) -> String {
        use TypeError::*;

        match self {
            UnboundId { .. } => "variable not found".to_owned(),
            UnboundFunc { .. } => "function not found".to_owned(),
            TypeMismatch { expected, .. } => format!("expected {expected}"),
            WrongNumArgs { expected, .. } => format!("expected {expected} arguments"),
            MissingMain => "main() not found".to_owned(),
            MainDoesNotReturnUnit => "expected type ()".to_owned(),
            MainTakesArgs => "main() takes no arguments".to_owned(),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
