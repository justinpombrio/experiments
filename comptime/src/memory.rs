use crate::ast::{Func, Id};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory Error: overwriting addr {0:0x} with {1}.")]
    Overwrite(u32, String),
    #[error("Memory Error: Stack underflow!")]
    StackUnderflow,
    #[error("Memory Error: No stack frame to bind local variable in.")]
    NoStackFrame,
    #[error("Memory Error: addr {addr:0x} contains {actual}, not {expected}")]
    InvalidRead {
        addr: u32,
        actual: &'static str,
        expected: &'static str,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Addr(u32);

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Unit,
    Int(i32),
    Ptr(Addr),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "()",
            Value::Int(_) => "Int",
            Value::Ptr(_) => "Ptr",
        }
    }
}

#[derive(Debug)]
pub enum HeapValue<'a> {
    Uninit,
    Free,
    Func(&'a Func),
    Array(Vec<Value>),
}

impl<'a> HeapValue<'a> {
    fn type_name(&self) -> &'static str {
        use HeapValue::*;

        match self {
            Uninit => "UninitializedMemory",
            Free => "FreedMemory",
            Func(_) => "Function",
            Array(_) => "Array",
        }
    }
}

struct Frame(Vec<(Id, Value)>);

impl Frame {
    fn new() -> Frame {
        Frame(Vec::new())
    }

    fn push(&mut self, id: Id, val: Value) {
        self.0.push((id, val));
    }

    fn get(&self, id: &Id) -> Option<Value> {
        for (key, val) in self.0.iter().rev() {
            if key == id {
                return Some(*val);
            }
        }
        None
    }
}

pub struct Memory<'a> {
    globals: Frame,
    stack: Vec<Frame>,
    heap: Vec<HeapValue<'a>>,
}

impl<'a> Memory<'a> {
    pub fn new() -> Memory<'a> {
        Memory {
            globals: Frame::new(),
            stack: Vec::new(),
            // Reserve addr 0
            heap: vec![HeapValue::Uninit],
        }
    }

    pub fn alloc(&mut self) -> Addr {
        let addr = Addr(self.heap.len() as u32);
        self.heap.push(HeapValue::Uninit);
        addr
    }

    pub fn free(&mut self, addr: Addr) {
        self.heap[addr.0 as usize] = HeapValue::Free;
    }

    fn write(&mut self, addr: Addr, value: HeapValue<'a>) -> Result<(), MemoryError> {
        let old_value = &mut self.heap[addr.0 as usize];
        if !matches!(old_value, HeapValue::Uninit) {
            return Err(MemoryError::Overwrite(addr.0, format!("{:?}", value)));
        }
        *old_value = value;
        Ok(())
    }

    pub fn write_array(&mut self, addr: Addr, array: Vec<Value>) -> Result<(), MemoryError> {
        self.write(addr, HeapValue::Array(array))
    }

    pub fn write_func(&mut self, addr: Addr, func: &'a Func) -> Result<(), MemoryError> {
        self.write(addr, HeapValue::Func(func))
    }

    pub fn read_func(&self, addr: Addr) -> Result<&'a Func, MemoryError> {
        let val = &self.heap[addr.0 as usize];
        if let HeapValue::Func(func) = val {
            Ok(func)
        } else {
            Err(MemoryError::InvalidRead {
                addr: addr.0,
                expected: "Function",
                actual: val.type_name(),
            })
        }
    }

    pub fn push_stack_frame(&mut self) {
        self.stack.push(Frame::new())
    }

    pub fn pop_stack_frame(&mut self) -> Result<(), MemoryError> {
        if self.stack.pop().is_none() {
            Err(MemoryError::StackUnderflow)
        } else {
            Ok(())
        }
    }

    pub fn bind_local(&mut self, id: &Id, val: Value) -> Result<(), MemoryError> {
        if let Some(frame) = self.stack.last_mut() {
            frame.push(id.clone(), val);
            Ok(())
        } else {
            Err(MemoryError::NoStackFrame)
        }
    }

    pub fn get_local(&self, id: &Id) -> Option<Value> {
        self.stack.last().and_then(|frame| frame.get(id))
    }

    pub fn bind_global(&mut self, id: Id, val: Value) {
        self.globals.push(id, val);
    }

    pub fn get_global(&mut self, id: &Id) -> Option<Value> {
        self.globals.get(id)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Ptr(addr) => write!(f, "{:#x}", addr.0),
        }
    }
}
