use crate::rt_error::RtError;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Value {
    Unit,
    Num(i32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Num(n) => write!(f, "{}", n),
        }
    }
}

impl Value {
    pub fn typename(&self) -> &'static str {
        match self {
            Value::Unit => "Unit",
            Value::Num(_) => "Number",
        }
    }

    pub fn unwrap_num(self, context: &'static str) -> Result<i32, RtError> {
        if let Value::Num(n) = self {
            Ok(n)
        } else {
            Err(RtError::err_tc("Number", &self, context))
        }
    }
}
