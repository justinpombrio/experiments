use crate::ast::{Expr, Func, FuncType, Id, Located, ParamMode, Prog, Type};
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

    fn lookup(&self, id: &str) -> Option<Type> {
        for (x, ty) in &self.0 {
            if x == id {
                return Some(ty.clone());
            }
        }
        None
    }
}

struct TypeChecker<'a> {
    prog: &'a Prog,
    ct_env: TypeEnv,
    rt_env: TypeEnv,
}

impl<'a> TypeChecker<'a> {
    fn new(prog: &Prog) -> TypeChecker {
        let mut rt_env = TypeEnv::new();
        for func in &prog.funcs {
            rt_env.push(func.inner.name.inner.clone(), func_type(&func.inner));
        }

        TypeChecker {
            prog,
            rt_env,
            ct_env: TypeEnv::new(),
        }
    }

    fn check_all(&mut self) -> Result<(), TypeError> {
        for func in &self.prog.funcs {
            self.check_func(func)?;
        }
        self.check_expr(&self.prog.main)?;
        Ok(())
    }

    fn check_func(&mut self, func_loc: &Located<Func>) -> Result<(), TypeError> {
        let func = &func_loc.inner;

        for param in &func.params {
            let param = &param.inner;
            self.env(param.mode)
                .push(param.id.clone(), param.ty.clone());
        }
        self.expect_expr(&func.body, func.returns.clone())?;
        for param in &func.params {
            self.env(param.inner.mode).pop();
        }

        Ok(())
    }

    fn check_id(&mut self, mode: ParamMode, id_loc: &Located<Id>) -> Result<Type, TypeError> {
        let id = &id_loc.inner;
        match self.env(mode).lookup(id) {
            Some(ty) => Ok(ty.to_owned()),
            None => Err(TypeError::UnboundId(id_loc.clone())),
        }
    }

    fn check_expr(&mut self, expr_loc: &Located<Expr>) -> Result<Type, TypeError> {
        let expr = &expr_loc.inner;

        match expr {
            Expr::Unit => Ok(Type::Unit),
            Expr::Int(_) => Ok(Type::Int),
            Expr::Id(mode, id) => self.check_id(*mode, id),
            Expr::Sum(exprs) => {
                for expr in exprs {
                    self.expect_expr(expr, Type::Int)?;
                }
                Ok(Type::Int)
            }
            Expr::Let(mode, id_loc, binding_loc, body_loc) => {
                let binding_ty = self.check_expr(binding_loc)?;
                self.env(*mode).push(id_loc.inner.clone(), binding_ty);
                self.check_expr(body_loc)
            }
            Expr::Call(func, args) => {
                let func_ty = self.expect_func(func)?;
                if args.len() != func_ty.params.len() {
                    return Err(TypeError::WrongNumArgs {
                        loc: func.loc,
                        expected: func_ty.params.len(),
                        actual: args.len(),
                    });
                }
                for (arg, param) in args.iter().zip(func_ty.params.iter()) {
                    let actual_ty = self.check_expr(arg)?;
                    let expected_ty = param;
                    if &actual_ty != expected_ty {
                        return Err(TypeError::TypeMismatch {
                            loc: func.loc,
                            expected: expected_ty.clone(),
                            actual: actual_ty,
                        });
                    }
                }
                Ok(func_ty.returns.as_ref().clone())
            }
        }
    }

    fn expect_func(&mut self, expr_loc: &Located<Expr>) -> Result<FuncType, TypeError> {
        let ty = self.check_expr(expr_loc)?;
        if let Type::Func(func_ty) = ty {
            Ok(func_ty)
        } else {
            Err(TypeError::ExpectedFunction {
                actual: ty,
                loc: expr_loc.loc,
            })
        }
    }

    fn expect_expr(
        &mut self,
        expr_loc: &Located<Expr>,
        expected_ty: Type,
    ) -> Result<(), TypeError> {
        let actual_ty = self.check_expr(expr_loc)?;
        if actual_ty == expected_ty {
            Ok(())
        } else {
            Err(TypeError::TypeMismatch {
                expected: expected_ty.to_owned(),
                actual: actual_ty.to_owned(),
                loc: expr_loc.loc,
            })
        }
    }

    fn env(&mut self, mode: ParamMode) -> &mut TypeEnv {
        match mode {
            ParamMode::Runtime => &mut self.rt_env,
            ParamMode::Comptime => &mut self.ct_env,
        }
    }
}

fn func_type(func: &Func) -> Type {
    Type::Func(FuncType {
        params: func
            .params
            .iter()
            .map(|param| param.inner.ty.clone())
            .collect(),
        returns: Box::new(func.returns.clone()),
    })
}
