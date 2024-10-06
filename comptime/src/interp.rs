use crate::expr::Expr;
use crate::rt_error::RtError;
use crate::value::Value;

pub fn interp_expr(expr: &Expr) -> Result<Value, RtError> {
    match expr {
        Expr::Num(n) => Ok(Value::Num(*n)),
        Expr::Add(exprs) => {
            let mut sum = 0;
            for expr in exprs {
                let n = interp_expr(expr)?.unwrap_num("addition")?;
                sum += n;
            }
            Ok(Value::Num(sum))
        }
    }
}
