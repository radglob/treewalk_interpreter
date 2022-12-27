use crate::token::{Literal,Token};

#[derive(Debug,Clone)]
pub enum Expr {
    Literal(Literal),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Box<Vec<Expr>>),
    Grouping(Box<Expr>),
    Variable(Token)
}
