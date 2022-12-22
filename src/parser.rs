use std::error::Error;

use crate::error::ParserError;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::TokenType::{self, *};
use crate::token::{Literal, Token};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn default() -> Self {
        Self {
            tokens: vec![],
            current: 0,
        }
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0
        }
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.comparison()?;
        while self.matches(vec![BangEqual, EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn matches(&mut self, types: Vec<TokenType>) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.term()?;
        while self.matches(vec![Greater, GreaterEqual, Less, LessEqual, Percent]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.factor()?;

        while self.matches(vec![Minus, Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.unary()?;

        while self.matches(vec![Slash, Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.matches(vec![Bang, Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.matches(vec![False]) {
            return Ok(Expr::Literal(Literal::False));
        }
        if self.matches(vec![True]) {
            return Ok(Expr::Literal(Literal::True));
        }
        if self.matches(vec![Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }
        if self.matches(vec![Number, String]) {
            let l = self.previous().literal.unwrap();
            return Ok(Expr::Literal(l));
        }
        if self.matches(vec![LeftParen]) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(ParserError::new(
            self.peek(),
            "Expect expression".to_string(),
        ))
    }

    fn consume(&mut self, t: TokenType, message: &str) -> Result<Token, ParserError> {
        if self.check(t) {
            return Ok(self.advance());
        }

        Err(ParserError::new(self.peek(), message.to_string()))
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == Semicolon { return; }
            match self.peek().token_type {
                Class | For | Fun | If | Print | Return | Var | While => return,
                _ => ()
            }
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Box<dyn Error>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            let statement = self.statement()?;
            statements.push(statement);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        if self.matches(vec![Print]) { return self.print_statement() }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Box<dyn Error>> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }
}
