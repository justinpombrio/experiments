use crate::ast::{Expr, Func, Id, Loc, Located, Phase, Prog};
use crate::eval_error::{EvalError, EvalErrorCase};
use crate::memory::{Addr, Memory, MemoryError, Value};

struct Interpreter<'a> {
    memory: Memory<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(prog: &'a Prog) -> Interpreter<'a> {
        let mut memory = Memory::new();
        for func in &prog.funcs {
            let addr = memory.alloc();
            try_memory(func.loc, memory.write_func(addr, &func.inner)).unwrap();
            memory.bind_global(func.inner.name.inner.clone(), Value::Ptr(addr));
        }

        Interpreter { memory }
    }

    fn id(&mut self, id: &Located<Id>) -> Result<Value, EvalError> {
        self.memory
            .get_local(&id.inner)
            .or_else(|| self.memory.get_global(&id.inner))
            .ok_or_else(|| EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::UnboundId(id.inner.clone()),
                loc: id.loc,
            })
    }

    fn eval_expr(&mut self, expr: &Located<Expr>) -> Result<Value, EvalError> {
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
            Expr::Id(id) => self.id(id),
            Expr::Let(id, binding, body) => {
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
            Expr::Comptime(_) => Err(EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::LeftoverComptime,
                loc: expr.loc,
            }),
        }
    }

    fn eval_func(&mut self, loc: Loc, func: Value) -> Result<&'a Func, EvalError> {
        let addr = unwrap_ptr(loc, func)?;
        try_memory(loc, self.memory.read_func(addr))
    }

    fn call(&mut self, loc: Loc, func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
        let func = self.eval_func(loc, func)?;
        check_num_args(loc, args.len(), func.params.len())?;
        self.memory.push_stack_frame();
        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            try_memory(loc, self.memory.bind_local(&param.inner.id, arg))?;
        }
        let result = self.eval_expr(&func.body)?;
        try_memory(loc, self.memory.pop_stack_frame())?;
        Ok(result)
    }
}

fn unwrap_int(loc: Loc, value: Value) -> Result<i32, EvalError> {
    if let Value::Int(n) = value {
        Ok(n)
    } else {
        Err(EvalError {
            phase: Phase::Runtime,
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
            phase: Phase::Runtime,
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
            phase: Phase::Runtime,
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
        phase: Phase::Runtime,
        error: EvalErrorCase::MemoryError(mem_err),
        loc,
    })
}

pub fn run_prog(prog: &Prog) -> Result<Value, EvalError> {
    let mut interp = Interpreter::new(prog);
    interp.eval_expr(&prog.main)
}
