use std::fs;
use std::io::{stderr, Write};
use std::process::exit;
use std::error::Error;

use crate::error::RuntimeError;
use crate::expr::Expr;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::Literal;
use crate::token::TokenType;

pub struct Interpreter {
    had_error: bool,
    had_runtime_error: bool
}

impl Interpreter {
    pub fn default() -> Self {
        Self { had_error: false, had_runtime_error: false }
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
        let mut scanner = Scanner::new(source);
        if let Err(err) = scanner.scan_tokens() {
            self.error(scanner.line as u32, err.to_string())?;
        }

        let mut parser = Parser::new(scanner.tokens);
        let expression = parser.parse().unwrap();

        if let Err(err) = self.interpret(expression) {
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
        writeln!(stderr(), "{}\n[line {}]", runtime_error.message, runtime_error.token.line)?;
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

    fn evaluate(&self, expr: Expr) -> Result<Literal, RuntimeError> {
        match expr {
            Expr::Literal(literal) => Ok(literal),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Unary(operator, right) => {
                let right = self.evaluate(*right).unwrap();
                match (operator.token_type, right.clone()) {
                    (TokenType::Minus, Literal::Number(n)) => Ok(Literal::Number(-n)),
                    (TokenType::Minus, _) => Err(RuntimeError::new(
                        operator,
                        "Operand must be a number.".to_string(),
                    )),
                    (TokenType::Bang, _) => {
                        let b = !self.is_truthy(&right);
                        if b {
                            Ok(Literal::True)
                        } else {
                            Ok(Literal::False)
                        }
                    }
                    _ => panic!(),
                }
            }
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(*left).unwrap();
                let right = self.evaluate(*right).unwrap();
                match (operator.token_type, left, right) {
                    (TokenType::Minus, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::Number(a - b))
                    }
                    (TokenType::Minus, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Slash, Literal::Number(a), Literal::Number(b)) => {
                        if b == 0.0 {
                            Err(RuntimeError::new(operator, "Cannot divide by zero".to_string()))
                        } else {
                            Ok(Literal::Number(a / b))
                        }
                    }
                    (TokenType::Slash, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Star, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::Number(a * b))
                    }
                    (TokenType::Star, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Plus, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::Number(a + b))
                    }
                    (TokenType::Plus, Literal::String(s1), Literal::String(s2)) => {
                        let mut s = String::from(s1);
                        s.push_str(&s2);
                        Ok(Literal::String(s))
                    }
                    (TokenType::Plus, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be two numbers or two strings.".to_string(),
                    )),
                    (TokenType::Percent, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::Number(a % b))
                    }
                    (TokenType::Percent, _, _) => Err(RuntimeError::new(
                            operator,
                            "Operands must be numbers".to_string()
                    )),
                    (TokenType::Greater, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::from(a > b))
                    }
                    (TokenType::Greater, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::GreaterEqual, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::from(a >= b))
                    }
                    (TokenType::GreaterEqual, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Less, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::from(a < b))
                    }
                    (TokenType::Less, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::LessEqual, Literal::Number(a), Literal::Number(b)) => {
                        Ok(Literal::from(a <= b))
                    }
                    (TokenType::LessEqual, _, _) => Err(RuntimeError::new(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::BangEqual, l1, l2) => Ok(Literal::from(!self.is_equal(&l1, &l2))),
                    (TokenType::EqualEqual, l1, l2) => Ok(Literal::from(self.is_equal(&l1, &l2))),

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

    fn interpret(&self, expr: Expr) -> Result<(), RuntimeError> {
        match self.evaluate(expr) {
            Ok(l) => {
                println!("{}", self.stringify(l));
                Ok(())
            }
            Err(err) => Err(err)
        }
    }

    fn stringify(&self, literal: Literal) -> String {
        match literal {
            Literal::Nil => "nil".to_string(),
            Literal::Number(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text = text[0 .. text.len() - 2].to_string();
                }
                text
            }
            Literal::String(s) => s,
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string()
        }
    }
}
