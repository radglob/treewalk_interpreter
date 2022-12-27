use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug,Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Token, Vec<Token>, Box<Vec<Stmt>>),
    Print(Expr),
    If(Expr, Box<Stmt>, Box<Option<Stmt>>),
    While(Option<Expr>, Box<Stmt>),
    Var(Token, Option<Expr>),
    Break(Token)
}
