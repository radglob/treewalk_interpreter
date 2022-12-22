use crate::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr)
}
