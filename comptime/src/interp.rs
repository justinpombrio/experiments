use crate::ast::{Expr, Func, Id, Prog, Value};
use crate::env::Env;
use crate::runtime_error::RuntimeError;
use std::collections::HashMap;

struct Interpreter<'a> {
    funcs: HashMap<Id, &'a Func>,
    env: Env,
}

impl<'a> Interpreter<'a> {
    pub fn new(prog: &'a Prog) -> Interpreter<'a> {
        let mut funcs = HashMap::new();
        for func in &prog.funcs {
            funcs.insert(func.name.clone(), func);
        }
        Interpreter {
            funcs,
            env: Env::new(),
        }
    }

    fn call(&mut self, func_name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if let Some(func) = self.funcs.get(func_name).copied() {
            if args.len() != func.params.len() {
                return Err(RuntimeError::WrongNumArgs {
                    func: func.name.to_owned(),
                    expected: func.params.len(),
                    actual: args.len(),
                });
            }
            for (i, arg) in args.into_iter().enumerate() {
                self.env.push(func.params[i].0.clone(), arg);
            }
            let result = self.eval_expr(&func.body)?;
            for _ in 0..func.params.len() {
                self.env.pop();
            }
            Ok(result)
        } else {
            Err(RuntimeError::ScopeBug {
                id: func_name.to_owned(),
            })
        }
    }

    fn id(&mut self, id: &str) -> Result<Value, RuntimeError> {
        self.env.take(id)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Unit => Ok(Value::Unit),
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Add(x, y) => {
                let x = self.eval_expr(x)?.unwrap_int("addition")?;
                let y = self.eval_expr(y)?.unwrap_int("addition")?;
                Ok(Value::Int(x + y))
            }
            Expr::Id(id) => self.id(id),
            Expr::Call(func_name, exprs) => {
                let mut args = Vec::new();
                for expr in exprs {
                    args.push(self.eval_expr(expr)?);
                }
                self.call(func_name, args)
            }
        }
    }
}

pub fn run(prog: &Prog) -> Result<Value, RuntimeError> {
    let mut interp = Interpreter::new(prog);
    interp.call("main", Vec::new())
}
