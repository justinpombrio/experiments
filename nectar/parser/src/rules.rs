use expr::Expr;



pub struct Rule<'s, 'g> {
    lhs: Expr<'s, 'g>,
    rhs: Expr<'s, 'g>,
}
