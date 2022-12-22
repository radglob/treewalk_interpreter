use std::error::Error;
use std::fs;
use std::io::{stderr, Write};
use std::process::exit;

use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::expr::Expr;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::stmt::Stmt;
use crate::token::Literal;
use crate::token::TokenType;

pub struct Interpreter {
    had_error: bool,
    had_runtime_error: bool,
    environment: Environment,
    repl: bool,
}

impl Interpreter {
    pub fn default() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            environment: Environment::new(),
            repl: false,
        }
    }

    pub fn run_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let contents: String = fs::read_to_string(path)?;
        self.run(contents)?;

        if self.had_error {
            exit(65)
        }

        if self.had_runtime_error {
            exit(70)
        }

        Ok(())
    }

    fn run(&mut self, source: String) -> Result<(), Box<dyn Error>> {
        self.repl = true;
        let mut scanner = Scanner::new(source);
        if let Err(err) = scanner.scan_tokens() {
            self.error(scanner.line as u32, err.to_string())?;
        }

        let mut parser = Parser::new(scanner.tokens);
        let statements = parser.parse()?;

        if let Err(err) = self.interpret(statements) {
            self.runtime_error(err)?;
        };

        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let mut input = String::new();
            print!("> ");
            let _ = std::io::stdout().flush();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    self.run(input)?;
                    self.had_error = false;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn error(&mut self, line: u32, message: String) -> Result<(), std::io::Error> {
        self.report(line, "".to_string(), message)?;
        Ok(())
    }

    fn runtime_error(&mut self, runtime_error: RuntimeError) -> Result<(), std::io::Error> {
        writeln!(
            stderr(),
            "{}\n[line {}]",
            runtime_error.message,
            runtime_error.token.line
        )?;
        self.had_runtime_error = true;
        Ok(())
    }

    fn report(
        &mut self,
        line: u32,
        location: String,
        message: String,
    ) -> Result<(), std::io::Error> {
        writeln!(stderr(), "[line {}] Error{}: {}", line, location, message)?;
        self.had_error = true;
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                match expr {
                    Expr::Assign(_, _) => {
                        self.evaluate(expr)?;
                    }
                    _ => {
                        let value = self.evaluate(expr)?;
                        if self.repl {
                            println!("{}", self.stringify(value))
                        }
                    }
                };
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", self.stringify(value));
                Ok(())
            }
            Stmt::Var(token, initializer) => {
                let mut value = None;
                if let Some(expr) = initializer {
                    value = Some(self.evaluate(expr)?)
                }

                match value {
                    None => {
                        return Err(RuntimeError::new(
                            token,
                            "Must assign value to new variable.".to_string(),
                        ))
                    }
                    Some(v) => self.environment.define(token.lexeme, v),
                }

                Ok(())
            }
            Stmt::While(condition, body) => {
                if let Some(condition) = condition {
                    let mut value = self.evaluate(condition.clone())?;
                    while self.is_truthy(&value) {
                        self.execute((*body).clone())?;
                        value = self.evaluate(condition.clone())?;
                    }
                }
                Ok(())
            }
            Stmt::Block(stmts) => self.evaluate_block(stmts),
            Stmt::If(condition, then_branch, else_branch) => {
                let value = self.evaluate(condition)?;
                if self.is_truthy(&value) {
                    self.execute(*then_branch)?
                } else if let Some(else_branch) = *else_branch {
                    self.execute(else_branch)?
                }
                Ok(())
            }
        }
    }

    fn evaluate_block(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        self.environment = Environment::with_enclosing(self.environment.clone());
        for stmt in stmts {
            self.execute(stmt)?;
        }

        if let Some(enclosing) = self.environment.enclosing.clone() {
            self.environment = *enclosing;
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Literal, RuntimeError> {
        match expr {
            Expr::Literal(literal) => Ok(literal),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Unary(operator, right) => {
                let right = self.evaluate(*right);
                match (operator.token_type, right.clone()) {
                    (TokenType::Minus, Ok(Literal::Number(n))) => Ok(Literal::Number(-n)),
                    (TokenType::Minus, _) => Err(RuntimeError::new(
                        operator,
                        "Operand must be a number.".to_string(),
                    )),
                    (TokenType::Bang, Ok(_)) => {
                        let b = !self.is_truthy(&right.unwrap());
                        if b {
                            Ok(Literal::True)
                        } else {
                            Ok(Literal::False)
                        }
                    }
                    (_, Err(err)) => Err(err),
                    _ => panic!(),
                }
            }
            Expr::Assign(token, value) => {
                let value = self.evaluate(*value)?;
                self.environment.assign(token, value.clone())?;
                Ok(value)
            }
            Expr::Variable(name) => self.environment.get(name),
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(*left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }

                self.evaluate(*right)
            }
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(*left);
                let right = self.evaluate(*right);
                match (operator.token_type, left, right) {
                    (TokenType::Minus, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a - b))
                    }
                    (TokenType::Minus, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Slash, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        if b == 0.0 {
                            Err(RuntimeError::new(
                                operator,
                                "Cannot divide by zero".to_string(),
                            ))
                        } else {
                            Ok(Literal::Number(a / b))
                        }
                    }
                    (TokenType::Slash, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Star, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a * b))
                    }
                    (TokenType::Star, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Plus, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a + b))
                    }
                    (TokenType::Plus, Ok(Literal::String(s1)), Ok(Literal::String(s2))) => {
                        let mut s = String::from(s1);
                        s.push_str(&s2);
                        Ok(Literal::String(s))
                    }
                    (TokenType::Plus, Ok(Literal::String(s1)), Ok(literal)) => {
                        let mut s = String::from(s1);
                        s.push_str(&literal.to_string());
                        Ok(Literal::String(s))
                    }
                    (TokenType::Plus, Ok(literal), Ok(Literal::String(s2))) => {
                        let mut s = literal.to_string();
                        s.push_str(&s2);
                        Ok(Literal::String(s))
                    }
                    (TokenType::Plus, Ok(l1), Ok(l2)) => {
                        println!("l1: {:?}, l2: {:?}", l1, l2);
                        Err(RuntimeError::new(
                            operator,
                            "Operands must be two numbers or two strings.".to_string(),
                        ))
                    }
                    (TokenType::Percent, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a % b))
                    }
                    (TokenType::Percent, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers".to_string(),
                    )),
                    (TokenType::Greater, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a > b))
                    }
                    (TokenType::Greater, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::GreaterEqual, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a >= b))
                    }
                    (TokenType::GreaterEqual, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Less, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a < b))
                    }
                    (TokenType::Less, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::LessEqual, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a <= b))
                    }
                    (TokenType::LessEqual, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::BangEqual, Ok(l1), Ok(l2)) => {
                        Ok(Literal::from(!self.is_equal(&l1, &l2)))
                    }
                    (TokenType::EqualEqual, Ok(l1), Ok(l2)) => {
                        Ok(Literal::from(self.is_equal(&l1, &l2)))
                    }
                    (_, Err(err), _) => Err(err),
                    (_, _, Err(err)) => Err(err),

                    _ => unimplemented!(),
                }
            }
        }
    }

    fn is_truthy(&self, v: &Literal) -> bool {
        match v {
            Literal::Nil | Literal::False => false,
            _ => true,
        }
    }

    fn is_equal(&self, a: &Literal, b: &Literal) -> bool {
        match (a, b) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Nil, _) => false,
            _ => a == b,
        }
    }

    fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.execute(stmt)?
        }
        Ok(())
    }

    fn stringify(&self, literal: Literal) -> String {
        match literal {
            Literal::Nil => "nil".to_string(),
            Literal::Number(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text = text[0..text.len() - 2].to_string();
                }
                text
            }
            Literal::String(s) => s,
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
        }
    }
}
