use crate::ast::{Expr, Func, Id, Located, Prog, Value};
use crate::env::Env;
use crate::runtime_error::RuntimeError;
use std::collections::HashMap;

struct Interpreter<'a> {
    funcs: HashMap<Id, &'a Located<Func>>,
    env: Env,
}

impl<'a> Interpreter<'a> {
    pub fn new(prog: &'a Prog) -> Interpreter<'a> {
        let mut funcs = HashMap::new();
        for func in &prog.funcs {
            funcs.insert(func.inner.name.inner.clone(), func);
        }
        Interpreter {
            funcs,
            env: Env::new(),
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
            for (i, arg) in args.into_iter().enumerate() {
                self.env.push(func.params[i].id.clone(), arg);
            }
            let result = self.eval_expr(&func.body)?;
            for _ in 0..func.params.len() {
                self.env.pop();
            }
            Ok(result)
        } else {
            Err(RuntimeError::UnboundId(func_id.clone()))
        }
    }

    fn id(&mut self, id: &Located<Id>) -> Result<Value, RuntimeError> {
        self.env.take(id)
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

pub fn run_prog(prog: &Prog) -> Result<Value, RuntimeError> {
    let mut interp = Interpreter::new(prog);
    interp.eval_expr(&prog.main)
}
