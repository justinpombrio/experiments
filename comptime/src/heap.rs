use crate::ast::Id;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory Error: overwriting addr {0} with {1}")]
    Overwrite(u32, String),
    #[error("Stack underflow!")]
    StackUnderflow,
    #[error("Stack value index {0} out of bounds")]
    InvalidStackValueIndex(usize),
    #[error("Constant index {0} out of bounds")]
    InvalidConstantIndex(usize),
}

#[derive(Debug)]
pub struct Addr(u32);

#[derive(Debug)]
pub enum StackValue {
    Unit,
    Int(i32),
    Ptr(Addr),
}

#[derive(Debug)]
pub enum HeapValue {
    Uninit,
    Free,
    Array(Vec<StackValue>),
}

struct StackFrame(HashMap<Id, StackValue>);

pub struct Memory {
    constants: HashMap<Id, StackValue>,
    stack: Vec<StackFrame>,
    heap: Vec<HeapValue>,
}

impl Memory {
    pub fn alloc(&mut self) -> Addr {
        let addr = Addr(self.heap.len() as u32);
        self.heap.push(HeapValue::Uninit);
        addr
    }

    pub fn free(&mut self, addr: Addr) {
        self.heap[addr.0 as usize] = HeapValue::Free;
    }

    fn write(&mut self, addr: Addr, value: HeapValue) -> Result<(), MemoryError> {
        let old_value = &mut self.heap[addr.0 as usize];
        if !matches!(old_value, HeapValue::Uninit) {
            return Err(MemoryError::Overwrite(addr.0, format!("{:?}", value)));
        }
        *old_value = value;
        Ok(())
    }

    pub fn write_array(&mut self, addr: Addr, array: Vec<StackValue>) -> Result<(), MemoryError> {
        self.write(addr, HeapValue::Array(array))
    }

    pub fn push_stack_frame(&mut self, values: Vec<StackValue>) {
        self.stack.push(StackFrame { values })
    }

    pub fn pop_stack_frame(&mut self) -> Result<(), MemoryError> {
        if self.stack.pop().is_none() {
            Err(MemoryError::StackUnderflow)
        } else {
            Ok(())
        }
    }

    pub fn lookup_stack(&self, index: usize) -> Result<StackValue, MemoryError> {
        if let Some(frame) = self.stack.last() {
        } else {
        }
    }

    pub fn add_constant(
}
