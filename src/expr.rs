use crate::token::{Literal,Token};
use crate::stmt::Stmt;

#[derive(Debug,Clone)]
pub enum Expr {
    Literal(Literal),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Lambda(Vec<Token>, Box<Vec<Stmt>>),
    Call(Box<Expr>, Token, Box<Vec<Expr>>),
    Grouping(Box<Expr>),
    Variable(Token),
    Empty
}
