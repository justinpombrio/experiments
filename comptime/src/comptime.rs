use crate::ast::{Expr, Func, Id, Loc, Located, ParamMode, Prog};
use crate::memory::{Addr, Memory, MemoryError, Value};
use crate::runtime_error::RuntimeError;

struct Compiler<'a> {
    memory: Memory<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(prog: &'a Prog) -> Compiler<'a> {
        let mut memory = Memory::new();
        for func in &prog.funcs {
            let addr = memory.alloc();
            try_memory(func.loc, memory.write_func(addr, &func.inner)).unwrap();
            memory.bind_global(func.inner.name.inner.clone(), Value::Ptr(addr));
        }

        Compiler { memory }
    }
}
