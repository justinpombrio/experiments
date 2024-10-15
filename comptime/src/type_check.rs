use crate::ast::{Expr, Func, Id, Prog, Type};
use crate::type_error::TypeError;

pub fn type_check(prog: &Prog) -> Result<(), TypeError> {
    let mut type_checker = TypeChecker::new(prog);
    type_checker.check_all()?;
    Ok(())
}

struct TypeEnv(Vec<(Id, Type)>);

impl TypeEnv {
    fn new() -> TypeEnv {
        TypeEnv(Vec::new())
    }

    fn push(&mut self, id: Id, ty: Type) {
        self.0.push((id, ty))
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    fn lookup(&self, id: &str) -> Option<&Type> {
        for (x, ty) in &self.0 {
            if x == id {
                return Some(ty);
            }
        }
        None
    }
}

struct TypeChecker<'a> {
    prog: &'a Prog,
    loc: String, // hacky, just stores the current function name
    env: TypeEnv,
}

impl<'a> TypeChecker<'a> {
    fn new(prog: &Prog) -> TypeChecker {
        TypeChecker {
            prog,
            loc: "[unknown]".to_owned(), // should never be visible
            env: TypeEnv::new(),
        }
    }

    fn check_all(&mut self) -> Result<(), TypeError> {
        for func in &self.prog.funcs {
            self.check_func(func)?;
        }

        let main = self
            .lookup_func("main")
            .ok_or_else(|| TypeError::MissingMain)?;
        if main.returns != Type::Unit {
            return Err(TypeError::MainDoesNotReturnUnit);
        }
        if !main.params.is_empty() {
            return Err(TypeError::MainTakesArgs);
        }
        Ok(())
    }

    fn check_func(&mut self, func: &Func) -> Result<(), TypeError> {
        for (id, ty) in &func.params {
            self.env.push(id.clone(), ty.clone());
        }

        self.loc = func.name.to_owned();
        self.expect_expr(&func.body, &func.returns)?;
        self.loc = "[unknown]".to_owned(); // should never be visible

        for _ in &func.params {
            self.env.pop();
        }

        Ok(())
    }

    fn check_id(&mut self, id: &str) -> Result<Type, TypeError> {
        match self.env.lookup(id) {
            Some(ty) => Ok(ty.to_owned()),
            None => Err(TypeError::UnboundId {
                id: id.to_owned(),
                loc: self.loc.clone(),
            }),
        }
    }

    fn lookup_func(&mut self, id: &str) -> Option<&'a Func> {
        self.prog.funcs.iter().find(|f| f.name == id)
    }

    fn expect_expr(&mut self, expr: &Expr, expected_ty: &Type) -> Result<(), TypeError> {
        let actual_ty = self.check_expr(expr)?;
        self.expect(&actual_ty, expected_ty)
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Unit => Ok(Type::Unit),
            Expr::Int(_) => Ok(Type::Int),
            Expr::Id(id) => self.check_id(id),
            Expr::Add(x, y) => {
                self.expect_expr(x, &Type::Int)?;
                self.expect_expr(y, &Type::Int)?;
                Ok(Type::Int)
            }
            Expr::Call(id, args) => {
                let func = self.lookup_func(id).ok_or_else(|| TypeError::UnboundFunc {
                    id: id.to_owned(),
                    loc: self.loc.clone(),
                })?;
                if args.len() != func.params.len() {
                    return Err(TypeError::WrongNumArgs {
                        func: func.name.clone(),
                        expected: func.params.len(),
                        actual: args.len(),
                        loc: self.loc.clone(),
                    });
                }
                for (i, (arg, param)) in args.iter().zip(func.params.iter()).enumerate() {
                    let actual_ty = self.check_expr(arg)?;
                    let expected_ty = &param.1;
                    if &actual_ty != expected_ty {
                        return Err(TypeError::BadArg {
                            func: func.name.clone(),
                            arg_index: i,
                            expected: expected_ty.clone(),
                            actual: actual_ty,
                            loc: self.loc.clone(),
                        });
                    }
                }
                Ok(func.returns.clone())
            }
        }
    }

    fn expect(&self, actual_ty: &Type, expected_ty: &Type) -> Result<(), TypeError> {
        if actual_ty == expected_ty {
            Ok(())
        } else {
            Err(TypeError::TypeMismatch {
                expected: expected_ty.to_owned(),
                actual: actual_ty.to_owned(),
                loc: self.loc.to_owned(),
            })
        }
    }
}
