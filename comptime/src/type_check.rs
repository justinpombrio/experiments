use crate::ast::{Expr, Func, Id, Located, Prog, Type};
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
    env: TypeEnv,
}

impl<'a> TypeChecker<'a> {
    fn new(prog: &Prog) -> TypeChecker {
        TypeChecker {
            prog,
            env: TypeEnv::new(),
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
            self.env.push(param.id.clone(), param.ty.clone());
        }
        self.expect_expr(&func.body, func.returns.clone())?;
        for _ in &func.params {
            self.env.pop();
        }

        Ok(())
    }

    fn check_id(&mut self, id_loc: &Located<Id>) -> Result<Type, TypeError> {
        let id = &id_loc.inner;
        match self.env.lookup(id) {
            Some(ty) => Ok(ty.to_owned()),
            None => Err(TypeError::UnboundId(id_loc.clone())),
        }
    }

    fn lookup_func(&mut self, id: &str) -> Option<&'a Located<Func>> {
        self.prog.funcs.iter().find(|f| f.inner.name.inner == id)
    }

    fn check_expr(&mut self, expr_loc: &Located<Expr>) -> Result<Type, TypeError> {
        let expr = &expr_loc.inner;

        match expr {
            Expr::Unit => Ok(Type::Unit),
            Expr::Int(_) => Ok(Type::Int),
            Expr::Id(id) => self.check_id(id),
            Expr::Sum(exprs) => {
                for expr in exprs {
                    self.expect_expr(expr, Type::Int)?;
                }
                Ok(Type::Int)
            }
            Expr::Call(id_loc, args) => {
                let id = &id_loc.inner;
                let func_loc = self
                    .lookup_func(id)
                    .ok_or_else(|| TypeError::UnboundFunc(id_loc.clone()))?;
                let func = &func_loc.inner;
                if args.len() != func.params.len() {
                    return Err(TypeError::WrongNumArgs {
                        callsite: id_loc.clone(),
                        defsite: func.name.clone(),
                        expected: func.params.len(),
                        actual: args.len(),
                    });
                }
                for (arg, param) in args.iter().zip(func.params.iter()) {
                    let actual_ty = self.check_expr(arg)?;
                    let expected_ty = &param.ty;
                    if &actual_ty != expected_ty {
                        return Err(TypeError::TypeMismatch {
                            expected: expected_ty.clone(),
                            actual: actual_ty,
                            loc: id_loc.loc,
                        });
                    }
                }
                Ok(func.returns.clone())
            }
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
}
