use crate::expr::Expr;
use crate::token::Token;

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    If(Expr, Box<Stmt>, Box<Option<Stmt>>),
    While(Expr, Box<Stmt>),
    Var(Token, Option<Expr>),
}
