use crate::ast::{Expr, Func, Id, Loc, Located, Prog};
use crate::memory::{Memory, MemoryError, Value};
use crate::runtime_error::RuntimeError;
use std::collections::HashMap;

struct Interpreter<'a> {
    funcs: HashMap<Id, &'a Located<Func>>,
    memory: Memory,
}

impl<'a> Interpreter<'a> {
    pub fn new(prog: &'a Prog) -> Interpreter<'a> {
        let mut funcs = HashMap::new();
        for func in &prog.funcs {
            funcs.insert(func.inner.name.inner.clone(), func);
        }
        Interpreter {
            funcs,
            memory: Memory::new(),
        }
    }

    fn call(&mut self, func_id: &Located<Id>, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if let Some(func_loc) = self.funcs.get(&func_id.inner).copied() {
            let func = &func_loc.inner;

            if args.len() != func.params.len() {
                return Err(RuntimeError::WrongNumArgs {
                    callsite: func_id.clone(),
                    defsite: func.name.clone(),
                    expected: func.params.len(),
                    actual: args.len(),
                });
            }
            self.memory.push_stack_frame();
            let params = func.params.iter().map(|param| &param.id);
            let args = args.into_iter();
            for (param, arg) in params.zip(args) {
                try_memory(func_id.loc, self.memory.bind_local(param, arg))?;
            }
            let result = self.eval_expr(&func.body)?;
            try_memory(func_id.loc, self.memory.pop_stack_frame())?;
            Ok(result)
        } else {
            Err(RuntimeError::UnboundId(func_id.clone()))
        }
    }

    fn id(&mut self, id: &Located<Id>) -> Result<Value, RuntimeError> {
        self.memory
            .lookup_stack(&id.inner)
            .ok_or_else(|| RuntimeError::UnboundId(id.clone()))
    }

    fn eval_expr(&mut self, expr: &Located<Expr>) -> Result<Value, RuntimeError> {
        match &expr.inner {
            Expr::Unit => Ok(Value::Unit),
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Sum(exprs) => {
                let mut sum = 0;
                for expr in exprs {
                    sum += self.eval_expr(expr)?.unwrap_int(expr)?;
                }
                Ok(Value::Int(sum))
            }
            Expr::Id(id) => self.id(id),
            Expr::Let(id, binding, body) => {
                let value = self.eval_expr(binding)?;
                try_memory(id.loc, self.memory.bind_local(&id.inner, value))?;
                self.eval_expr(body)
            }
            Expr::Call(func_id, exprs) => {
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.eval_expr(expr)?);
                }
                self.call(func_id, args)
            }
        }
    }
}

fn try_memory<T>(loc: Loc, result: Result<T, MemoryError>) -> Result<T, RuntimeError> {
    result.map_err(|error| RuntimeError::MemoryError { error, loc })
}

pub fn run_prog(prog: &Prog) -> Result<Value, RuntimeError> {
    let mut interp = Interpreter::new(prog);
    interp.eval_expr(&prog.main)
}
