use crate::ast::{Id, Loc, Type};
use crate::pretty_error::PrettyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Variable {id} not found")]
    UnboundId { id: Id, loc: Loc },

    #[error("Function {id} not found")]
    UnboundFunc { id: Id, loc: Loc },

    #[error("Expected type {expected} but found {actual}")]
    TypeMismatch {
        expected: Type,
        actual: Type,
        loc: Loc,
    },

    #[error("Wrong number of arguments passed to {func}. Expected {expected} args, but received {actual}.")]
    WrongNumArgs {
        func: Id,
        expected: usize,
        actual: usize,
        loc: Loc,
    },

    #[error("Expected type {expected} but received {actual}, for arg number {arg_index} in call to {func}")]
    BadArg {
        func: Id,
        arg_index: usize,
        expected: Type,
        actual: Type,
        loc: Loc,
    },

    #[error("Missing main() function.")]
    MissingMain,

    #[error("The main() function's return type must be ().")]
    MainDoesNotReturnUnit,

    #[error("The main() function must not take any arguments.")]
    MainTakesArgs,
}

impl PrettyError for TypeError {
    fn kind(&self) -> &'static str {
        "type error"
    }

    fn loc(&self) -> Option<Loc> {
        use TypeError::*;

        match self {
            UnboundId { loc, .. }
            | UnboundFunc { loc, .. }
            | TypeMismatch { loc, .. }
            | WrongNumArgs { loc, .. }
            | BadArg { loc, .. } => Some(*loc),
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
            BadArg {
                expected, actual, ..
            } => format!("expected {expected}, found {actual}"),
            MissingMain => "main() not found".to_owned(),
            MainDoesNotReturnUnit => "expected type ()".to_owned(),
            MainTakesArgs => "main() takes no arguments".to_owned(),
        }
    }

    fn long_message(&self) -> String {
        format!("{}", self)
    }
}
