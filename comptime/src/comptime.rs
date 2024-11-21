// TODO: temporary
#![allow(unused)]

use crate::ast::{Id, Loc, Located, Prog};
use crate::eval_error::{EvalError, EvalErrorCase, Phase};
use crate::memory::{Addr, Memory, MemoryError, Value};

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

    fn comp_id(&mut self, id: &Located<Id>) -> Result<Value, EvalError> {
        self.memory
            .get_local(&id.inner)
            .or_else(|| self.memory.get_global(&id.inner))
            .ok_or_else(|| EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::UnboundId(id.inner.clone()),
                loc: id.loc,
            })
    }
}

fn unwrap_int(loc: Loc, value: Value) -> Result<i32, EvalError> {
    if let Value::Int(n) = value {
        Ok(n)
    } else {
        Err(EvalError {
            phase: Phase::Comptime,
            loc,
            error: EvalErrorCase::TypeMismatch {
                expected: "Int",
                actual: value.type_name(),
            },
        })
    }
}

fn check_num_args(loc: Loc, num_args: usize, num_params: usize) -> Result<(), EvalError> {
    if num_args != num_params {
        Err(EvalError {
            phase: Phase::Comptime,
            error: EvalErrorCase::WrongNumArgs {
                expected: num_params,
                actual: num_args,
            },
            loc,
        })
    } else {
        Ok(())
    }
}

fn unwrap_ptr(loc: Loc, value: Value) -> Result<Addr, EvalError> {
    if let Value::Ptr(addr) = value {
        Ok(addr)
    } else {
        Err(EvalError {
            phase: Phase::Comptime,
            error: EvalErrorCase::TypeMismatch {
                expected: "Pointer",
                actual: value.type_name(),
            },
            loc,
        })
    }
}

fn try_memory<T>(loc: Loc, result: Result<T, MemoryError>) -> Result<T, EvalError> {
    result.map_err(|mem_err| EvalError {
        phase: Phase::Comptime,
        error: EvalErrorCase::MemoryError(mem_err),
        loc,
    })
}
