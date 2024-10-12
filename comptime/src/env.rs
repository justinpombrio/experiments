use crate::ast::{Value, Var};
use crate::rt_error::RtError;

pub struct Env {
    entries: Vec<(Var, Option<Value>)>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, var: Var, val: Value) {
        self.entries.push((var, Some(val)));
    }

    pub fn pop(&mut self) {
        self.entries.pop();
    }

    pub fn take(&mut self, var: &str) -> Result<Value, RtError> {
        for (x, val) in self.entries.iter_mut().rev() {
            if x == var {
                if let Some(val) = val.take() {
                    return Ok(val);
                }
            }
        }
        Err(RtError::err_var(var.to_owned()))
    }
}
