use crate::ast::{Id, Value};
use crate::runtime_error::RuntimeError;

pub struct Env {
    entries: Vec<(Id, Option<Value>)>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, id: Id, val: Value) {
        self.entries.push((id, Some(val)));
    }

    pub fn pop(&mut self) {
        self.entries.pop();
    }

    pub fn take(&mut self, id: &str) -> Result<Value, RuntimeError> {
        for (x, val) in self.entries.iter_mut().rev() {
            if x == id {
                if let Some(val) = val.take() {
                    return Ok(val);
                }
            }
        }
        Err(RuntimeError::err_id(id.to_owned()))
    }
}
