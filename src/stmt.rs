use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Token, Vec<Token>, Box<Vec<Stmt>>),
    Print(Expr),
    Return(Token, Box<Option<Expr>>),
    If(Expr, Box<Stmt>, Box<Option<Stmt>>),
    While(Expr, Box<Stmt>),
    Var(Token, Option<Expr>),
    Break(Token),
}
