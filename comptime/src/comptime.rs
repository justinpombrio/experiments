// TODO: temporary
#![allow(unused)]

use crate::ast::{Expr, Func, Id, Loc, Located, Phase, Prog, Type};
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
            memory.bind_global(func.inner.name.inner.clone(), Value::ptr(addr));
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
                let ty = ty.as_ref().expect("TC didn't annotate #expr");
                let lowered_expr = self.lower(expr.loc, value, ty)?;
                *expr = Box::new(Located {
                    loc: expr.loc,
                    inner: lowered_expr,
                });
                Ok(())
            }
        }
    }

    fn ct_expr(&mut self, expr: &mut Located<Expr>) -> Result<Value, EvalError> {
        match &mut expr.inner {
            Expr::Unit => Ok(Value::unit()),
            Expr::Int(n) => Ok(Value::int(*n)),
            Expr::Sum(exprs) => {
                let mut sum = 0;
                for expr in exprs {
                    sum += self.ct_expr(expr)?.unwrap_int(Phase::Comptime, expr.loc)?;
                }
                Ok(Value::int(sum))
            }
            Expr::Id(id) => self.ct_id(id),
            Expr::Let(id, binding, body) => {
                let value = self.ct_expr(binding)?;
                try_memory(id.loc, self.memory.bind_local(&id.inner, value))?;
                self.ct_expr(body)
            }
            Expr::Call(func_expr, exprs) => {
                let func = self.ct_expr(func_expr)?;
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.ct_expr(expr)?);
                }
                self.call(func_expr.loc, func, args)
            }
            Expr::Comptime(_) => Err(EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::NestedComptime,
                loc: expr.loc,
            }),
        }
    }

    /*
    fn ct_expr(&mut self, expr: &mut Located<Expr>) -> Result<Value, EvalError> {
        match &mut expr.inner {
            Expr::Unit => Ok(Value::unit()),
            Expr::Int(n) => Ok(Value::int(*n)),
            Expr::Sum(exprs) => {
                let mut sum = 0;
                for expr in exprs {
                    sum += self.ct_expr(expr)?.unwrap_int(Phase::Comptime, expr.loc)?;
                }
                Ok(Value::int(sum))
            }
            Expr::Id(id) => self.ct_id(id),
            Expr::Let(id, binding, body) => {
                let value = self.ct_expr(binding)?;
                try_memory(id.loc, self.memory.bind_local(&id.inner, value))?;
                self.ct_expr(body)
            }
            Expr::Call(func_expr, exprs) => {
                let func = self.ct_expr(func_expr)?;
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.ct_expr(expr)?);
                }
                self.call(func_expr.loc, func, args)
            }
            Expr::Comptime(_) => Err(EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::NestedComptime,
                loc: expr.loc,
            }),
        }
    }
    */

    fn ct_func(&mut self, loc: Loc, func: Value) -> Result<&'a Func, EvalError> {
        let addr = func.unwrap_ptr(Phase::Comptime, loc)?;
        try_memory(loc, self.memory.read_func(addr))
    }

    fn call(&mut self, loc: Loc, func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
        let func = self.ct_func(loc, func)?;
        check_num_args(loc, args.len(), func.params.len())?;
        self.memory.push_stack_frame();
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            try_memory(loc, self.memory.bind_local(&param.inner.id, arg))?;
        }
        let result = self.ct_expr(&mut func.body)?;
        try_memory(loc, self.memory.pop_stack_frame())?;
        Ok(result)
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
