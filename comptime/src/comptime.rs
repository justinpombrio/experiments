// TODO: temporary
#![allow(unused)]

use crate::ast::{Expr, Id, Loc, Located, Phase, Prog};
use crate::eval_error::{EvalError, EvalErrorCase};
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

    fn ct_id(&mut self, id: &Located<Id>) -> Result<Value, EvalError> {
        self.memory
            .get_local(&id.inner)
            .or_else(|| self.memory.get_global(&id.inner))
            .ok_or_else(|| EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::UnboundId(id.inner.clone()),
                loc: id.loc,
            })
    }

    fn rt_expr(&mut self, expr: &mut Located<Expr>) -> Result<(), EvalError> {
        match &mut expr.inner {
            Expr::Unit | Expr::Int(_) | Expr::Id(_) => Ok(()),
            Expr::Sum(exprs) => {
                for expr in exprs {
                    self.rt_expr(expr)?;
                }
                Ok(())
            }
            Expr::Let(_id, binding, body) => {
                self.rt_expr(binding)?;
                self.rt_expr(body)
            }
            Expr::Call(func, args) => {
                self.rt_expr(func)?;
                for arg in args {
                    self.rt_expr(arg)?;
                }
                Ok(())
            }
            Expr::Comptime(expr) => {
                let value = self.ct_expr(expr)?;
                *expr = self.lower(value)?;
                Ok(())
            }
        }
    }

    fn ct_expr(&mut self, expr: &mut Located<Expr>) -> Result<Value, EvalError> {}

    fn run_expr(&mut self, expr: Located<Expr>) -> Result<Expr, EvalError> {
        match expr.inner {
            Expr::Unit => Ok(Expr::Unit),
            Expr::Int(n) => Ok(Expr::Int(n)),
            Expr::Sum(exprs) => {
                let compiled_exprs = Vec::new();
                for expr in exprs {
                    compiled_exprs.push(self.run_expr(expr)?);
                }
                Ok(Expr::Sum(compiled_exprs))
            }
            Expr::Id(mode, id) => {
                check_is_runtime_mode(id.loc, *mode)?;
                self.id(id)
            }
            Expr::Let(mode, id, binding, body) => {
                check_is_runtime_mode(expr.loc, *mode)?;
                let value = self.eval_expr(binding)?;
                try_memory(id.loc, self.memory.bind_local(&id.inner, value))?;
                self.eval_expr(body)
            }
            Expr::Call(func_expr, exprs) => {
                let func = self.eval_expr(func_expr)?;
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.eval_expr(expr)?);
                }
                self.call(func_expr.loc, func, args)
            }
        }
    }

    fn comp_expr(&mut self, expr: &Located<Expr>) -> Result<Value, EvalError> {
        match &expr.inner {
            Expr::Unit => Ok(Value::Unit),
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Sum(exprs) => {
                let mut sum = 0;
                for expr in exprs {
                    sum += unwrap_int(expr.loc, self.eval_expr(expr)?)?;
                }
                Ok(Value::Int(sum))
            }
            Expr::Id(mode, id) => {
                check_is_runtime_mode(id.loc, *mode)?;
                self.id(id)
            }
            Expr::Let(mode, id, binding, body) => {
                check_is_runtime_mode(expr.loc, *mode)?;
                let value = self.eval_expr(binding)?;
                try_memory(id.loc, self.memory.bind_local(&id.inner, value))?;
                self.eval_expr(body)
            }
            Expr::Call(func_expr, exprs) => {
                let func = self.eval_expr(func_expr)?;
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.eval_expr(expr)?);
                }
                self.call(func_expr.loc, func, args)
            }
        }
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
