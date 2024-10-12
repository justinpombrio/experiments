use crate::ast::{Expr, Func, Prog, Value, Var};
use crate::env::Env;
use crate::rt_error::RtError;
use std::collections::HashMap;

struct Interpreter<'a> {
    funcs: &'a HashMap<Var, Func>,
    env: Env,
}

impl<'a> Interpreter<'a> {
    pub fn new(funcs: &'a HashMap<Var, Func>) -> Interpreter<'a> {
        Interpreter {
            funcs,
            env: Env::new(),
        }
    }

    fn call(&mut self, func_name: &str, args: Vec<Value>) -> Result<Value, RtError> {
        if let Some(func) = self.funcs.get(func_name) {
            if args.len() != func.params.len() {
                return Err(RtError::WrongNumArgs {
                    func: func.name.to_owned(),
                    expected: func.params.len(),
                    found: args.len(),
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
            Err(RtError::ScopeBug {
                var: func_name.to_owned(),
            })
        }
    }

    fn var(&mut self, var: &str) -> Result<Value, RtError> {
        self.env.take(var)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RtError> {
        match expr {
            Expr::Unit => Ok(Value::Unit),
            Expr::Int(n) => Ok(Value::Int(*n)),
            Expr::Add(x, y) => {
                let x = self.eval_expr(x)?.unwrap_int("addition")?;
                let y = self.eval_expr(y)?.unwrap_int("addition")?;
                Ok(Value::Int(x + y))
            }
            Expr::Id(var) => self.var(var),
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

pub fn run(prog: Prog) -> Result<Value, RtError> {
    let mut funcs = HashMap::new();
    for func in prog.funcs {
        funcs.insert(func.name.clone(), func);
    }
    let mut interp = Interpreter::new(&funcs);
    interp.call("main", Vec::new())
}
