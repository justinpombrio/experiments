use crate::ast::{Id, Type};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeError {
    #[error("Variable {id} not found, in {loc}")]
    UnboundId { id: Id, loc: String },

    #[error("Function {id} not found, in call in {loc}")]
    UnboundFunc { id: Id, loc: String },

    #[error("Type error: expected {expected} but found {actual}, in {loc}")]
    TypeMismatch {
        expected: Type,
        actual: Type,
        loc: String,
    },

    #[error("Wrong number of arguments passed to {func}. Expected {expected} args, but received {actual}.")]
    WrongNumArgs {
        func: Id,
        expected: usize,
        actual: usize,
        loc: String,
    },

    #[error("Type error: expected {expected} but found {actual}, for arg number {arg_index} in call to {func}, in {loc}")]
    BadArg {
        func: Id,
        arg_index: usize,
        expected: Type,
        actual: Type,
        loc: String,
    },

    #[error("Missing main() function.")]
    MissingMain,

    #[error("The main() function's return type must be ().")]
    MainDoesNotReturnUnit,

    #[error("The main() function must not take any arguments.")]
    MainTakesArgs,
}
