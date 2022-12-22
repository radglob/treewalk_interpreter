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
        Self { tokens, current: 0 }
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.or()?;
        if self.matches(vec![Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, Box::new(value)));
            }

            return Err(ParserError::new(
                equals,
                "Invalid assignment target.".to_string(),
            ));
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.and()?;
        while self.matches(vec![Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.equality()?;

        while self.matches(vec![And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
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
        if self.matches(vec![Identifier]) {
            return Ok(Expr::Variable(self.previous()));
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
            if self.previous().token_type == Semicolon {
                return;
            }
            match self.peek().token_type {
                Class | For | Fun | If | Print | Return | Var | While => return,
                _ => (),
            }
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements = vec![];
        while !self.is_at_end() {
            let statement = self.declaration()?;
            statements.push(statement);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        if self.matches(vec![Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(Identifier, "Expect variable name.")?;

        let mut initializer = None;
        if self.matches(vec![Equal]) {
            initializer = Some(self.expression()?)
        }

        self.consume(Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.matches(vec![For]) {
            return self.for_statement();
        }

        if self.matches(vec![Break]) {
            return self.break_statement();
        }

        if self.matches(vec![If]) {
            return self.if_statement();
        }

        if self.matches(vec![While]) {
            return self.while_statement();
        }

        if self.matches(vec![Print]) {
            return self.print_statement();
        }

        if self.matches(vec![LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "Expect '(' after 'for'.")?;

        let mut initializer: Option<Stmt> = None;
        if self.matches(vec![Semicolon]) {
        } else if self.matches(vec![Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Expr> = None;
        if !self.check(Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(Semicolon, "Expect ';' after loop condition.")?;

        let mut increment = None;
        if !self.check(RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(RightParen, "Expect ')' after for clauses.")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        if let None = condition {
            condition = Some(Expr::Literal(Literal::True));
        }

        body = Stmt::While(condition, Box::new(body));

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While(Some(condition), Box::new(body)))
    }

    fn break_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(Semicolon, "Expect ';' after break keyword.")?;
        let token = Token::new(TokenType::Break, "break".to_string(), None, self.current as u32);
        Ok(Stmt::Break(token))
    }

    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume(LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.matches(vec![Else]) {
            else_branch = Some(self.statement()?)
        }

        Ok(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut stmts = vec![];
        while !self.check(RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?)
        }

        self.consume(RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }
}
