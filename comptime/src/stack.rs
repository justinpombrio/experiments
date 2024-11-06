use crate::ast::{Id, Located, Value};
use crate::runtime_error::RuntimeError;

pub struct Stack {
    entries: Vec<(Id, Option<Value>)>,
}

impl Stack {
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

    pub fn take(&mut self, id_loc: &Located<Id>) -> Result<Value, RuntimeError> {
        let id = &id_loc.inner;

        for (x, val) in self.entries.iter_mut().rev() {
            if x == id {
                if let Some(val) = val.take() {
                    return Ok(val);
                }
            }
        }
        Err(RuntimeError::UnboundId(id_loc.clone()))
    }
}
