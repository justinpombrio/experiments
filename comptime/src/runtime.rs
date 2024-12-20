use crate::ast::{Expr, Func, Id, Loc, Located, Phase, Prog};
use crate::eval_error::{EvalError, EvalErrorCase};
use crate::memory::{Memory, MemoryError, Value};

struct Interpreter<'a> {
    memory: Memory<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(prog: &'a Prog) -> Interpreter<'a> {
        let mut memory = Memory::new();
        for func in &prog.funcs {
            let addr = memory.alloc();
            try_memory(func.loc, memory.write_func(addr, &func.inner)).unwrap();
            memory.bind_global(func.inner.name.inner.clone(), Value::ptr(addr));
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
            Expr::Unit => Ok(Value::unit()),
            Expr::Int(n) => Ok(Value::int(*n)),
            Expr::Sum(exprs) => {
                let mut sum = 0;
                for expr in exprs {
                    sum += self.eval_expr(expr)?.unwrap_int(Phase::Runtime, expr.loc)?;
                }
                Ok(Value::int(sum))
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
            Expr::Comptime(_, _) => Err(EvalError {
                phase: Phase::Runtime,
                error: EvalErrorCase::LeftoverComptime,
                loc: expr.loc,
            }),
        }
    }

    fn eval_func(&mut self, loc: Loc, func: Value) -> Result<&'a Func, EvalError> {
        let addr = func.unwrap_ptr(Phase::Runtime, loc)?;
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
