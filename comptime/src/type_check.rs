use crate::ast::{Expr, Func, FuncType, Id, Loc, Located, Phase, Prog, Type};
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
        self.check_expr(Phase::Runtime, &self.prog.main)?;
        Ok(())
    }

    fn check_func(&mut self, func_loc: &Located<Func>) -> Result<(), TypeError> {
        let func = &func_loc.inner;

        for param in &func.params {
            let param = &param.inner;
            self.env(param.phase)
                .push(param.id.clone(), param.ty.clone());
        }
        let ty = self.check_expr(Phase::Runtime, &func.body)?;
        expect_type(func.body.loc, &ty, &func.returns)?;
        for param in &func.params {
            self.env(param.inner.phase).pop();
        }

        Ok(())
    }

    fn check_id(&mut self, phase: Phase, id_loc: &Located<Id>) -> Result<Type, TypeError> {
        let id = &id_loc.inner;
        match self.env(phase).lookup(id) {
            Some(ty) => Ok(ty.to_owned()),
            None => Err(TypeError::UnboundId(id_loc.clone())),
        }
    }

    fn check_expr(&mut self, phase: Phase, expr_loc: &Located<Expr>) -> Result<Type, TypeError> {
        let expr = &expr_loc.inner;

        match expr {
            Expr::Unit => Ok(Type::Unit),
            Expr::Int(_) => Ok(Type::Int),
            Expr::Id(id) => self.check_id(phase, id),
            Expr::Sum(exprs) => {
                for expr in exprs {
                    let ty = self.check_expr(phase, expr)?;
                    expect_type(expr.loc, &ty, &Type::Int)?;
                }
                Ok(Type::Int)
            }
            Expr::Let(id_loc, binding_loc, body_loc) => {
                let binding_ty = self.check_expr(phase, binding_loc)?;
                self.env(phase).push(id_loc.inner.clone(), binding_ty);
                self.check_expr(phase, body_loc)
            }
            Expr::Call(func, args) => {
                let func_ty = unwrap_func(func.loc, self.check_expr(phase, func)?)?;
                assert_num_args(func.loc, args.len(), func_ty.params.len())?;
                for (arg, param) in args.iter().zip(func_ty.params.iter()) {
                    let actual_ty = self.check_expr(phase, arg)?;
                    let expected_ty = param;
                    expect_type(arg.loc, &actual_ty, expected_ty)?;
                }
                Ok(func_ty.returns.as_ref().clone())
            }
            Expr::Comptime(expr) => {
                assert_not_in_comptime(expr.loc, phase)?;
                self.check_expr(Phase::Comptime, expr)
            }
        }
    }

    fn env(&mut self, phase: Phase) -> &mut TypeEnv {
        match phase {
            Phase::Runtime => &mut self.rt_env,
            Phase::Comptime => &mut self.ct_env,
        }
    }
}

fn assert_not_in_comptime(loc: Loc, phase: Phase) -> Result<(), TypeError> {
    match phase {
        Phase::Runtime => Ok(()),
        Phase::Comptime => Err(TypeError::NestedComptime(loc)),
    }
}

fn assert_num_args(
    loc: Loc,
    actual_num_args: usize,
    expected_num_args: usize,
) -> Result<(), TypeError> {
    if actual_num_args == expected_num_args {
        Ok(())
    } else {
        Err(TypeError::WrongNumArgs {
            expected: expected_num_args,
            actual: actual_num_args,
            loc,
        })
    }
}

fn unwrap_func(loc: Loc, ty: Type) -> Result<FuncType, TypeError> {
    if let Type::Func(func_ty) = ty {
        Ok(func_ty)
    } else {
        Err(TypeError::ExpectedFunction { actual: ty, loc })
    }
}

fn expect_type(loc: Loc, actual_ty: &Type, expected_ty: &Type) -> Result<(), TypeError> {
    if actual_ty == expected_ty {
        Ok(())
    } else {
        Err(TypeError::TypeMismatch {
            expected: expected_ty.to_owned(),
            actual: actual_ty.to_owned(),
            loc,
        })
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
