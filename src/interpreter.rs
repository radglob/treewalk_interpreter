use std::error::Error;
use std::fs;
use std::io::{stderr, Write};
use std::process::exit;
use std::collections::HashMap;

use crate::callable::Callable;
use crate::environment::Environment;
use crate::error::*;
use crate::expr::Expr;
use crate::lox_function::LoxFunction;
use crate::native_function::*;
use crate::parser::Parser;
use crate::resolver::{Resolver, Resolve};
use crate::scanner::Scanner;
use crate::stmt::Stmt;
use crate::token::Literal;
use crate::token::Token;
use crate::token::TokenType;

pub type InterpreterResult<T> = Result<T, RuntimeException>;

#[derive(Clone)]
pub struct Interpreter {
    had_error: bool,
    had_runtime_error: bool,
    pub environment: Environment,
    repl: bool,
    loop_count: u32,
    locals: HashMap<Expr, u32>
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut environment = Environment::new();
        let clock = Literal::NativeFunction(NativeFunction {
            name: "clock".to_string(),
            arity: 0,
            callable: clock,
        });
        environment.define("clock".to_string(), clock);
        Self {
            had_error: false,
            had_runtime_error: false,
            environment,
            repl: false,
            loop_count: 0,
            locals: HashMap::new()
        }
    }
}

impl Interpreter {
    pub fn new(environment: &Environment) -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            environment: Environment::with_enclosing(environment.clone()),
            loop_count: 0,
            repl: false,
            locals: HashMap::new()
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
        let mut scanner = Scanner::new(source);
        if let Err(err) = scanner.scan_tokens() {
            self.error(scanner.line as u32, err.to_string())?;
        }

        let mut parser = Parser::new(scanner.tokens);
        let statements = parser.parse();

        if self.had_error {
            return Ok(())
        }

        match statements {
            Err(err) => {
                parser.synchronize();
                self.parser_error(err)?
            }
            Ok(statements) => {
                let mut resolver = Resolver::new(self.clone());
                resolver.resolve(statements.clone());
                self.had_error = resolver.interpreter.had_error;

                if self.had_error {
                    return Ok(())
                }

                if let Err(err) = self.interpret(statements) {
                    if let RuntimeException::Base(err) = err {
                        self.runtime_error(err)?;
                    }
                };
            }
        }
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

    pub fn error(&mut self, line: u32, message: String) -> Result<(), std::io::Error> {
        self.report(line, "".to_string(), message)?;
        Ok(())
    }

    fn parser_error(&mut self, parser_error: ParserError) -> Result<(), std::io::Error> {
        writeln!(
            stderr(),
            "{}\n[line {}]",
            parser_error.message,
            parser_error.token.line
        )
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

    pub fn log_error(&mut self, token: Token, message: String) -> Result<(), std::io::Error> {
        if token.token_type == TokenType::Eof {
            self.report(token.line, "at end".to_string(), message)?;
        } else {
            self.report(token.line, format!(" at '{}'", token.lexeme), message)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> InterpreterResult<()> {
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
                        return Err(RuntimeException::base(
                            token,
                            "Must assign value to new variable.".to_string(),
                        ))
                    }
                    Some(v) => self.environment.define(token.lexeme, v),
                }

                Ok(())
            }
            Stmt::While(condition, body) => {
                let mut value = self.evaluate(condition.clone())?;
                self.loop_count += 1;
                while self.is_truthy(&value) {
                    match self.execute((*body).clone()) {
                        Ok(()) => (),
                        Err(err) => match err {
                            RuntimeException::Break => break,
                            _ => return Err(err),
                        },
                    }
                    value = self.evaluate(condition.clone())?;
                }
                self.loop_count -= 1;
                Ok(())
            }
            Stmt::Block(stmts) => self.evaluate_block(stmts),
            Stmt::If(condition, then_branch, else_branch) => {
                let value = self.evaluate(condition)?;
                if self.is_truthy(&value) {
                    self.execute(*then_branch)?;
                } else if let Some(else_branch) = *else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            Stmt::Break(token) => {
                if self.loop_count > 0 {
                    Err(RuntimeException::Break)
                } else {
                    Err(RuntimeException::base(
                        token,
                        "Expected to be within a loop.".to_string(),
                    ))
                }
            }
            Stmt::Function(name, params, body) => {
                let stmt = Stmt::Function(name.clone(), params, body);
                let function = Literal::LoxFunction(LoxFunction::new(
                    name.lexeme.clone(),
                    stmt,
                    self.environment.clone(),
                ));
                self.environment.define(name.lexeme, function);
                Ok(())
            }
            Stmt::Return(_keyword, value) => {
                let v = match *value {
                    Some(value) => Some(self.evaluate(value)?),
                    None => None,
                };

                Err(RuntimeException::Return(Return::new(v)))
            }
        }
    }

    pub fn resolve(&mut self, expr: Expr, depth: u32) {
        self.locals.insert(expr, depth);
    }

    pub fn evaluate_block(&mut self, stmts: Vec<Stmt>) -> InterpreterResult<()> {
        self.environment = Environment::with_enclosing(self.environment.clone());
        for stmt in stmts {
            self.execute(stmt)?;
        }

        if let Some(enclosing) = self.environment.enclosing.clone() {
            self.environment = *enclosing;
        }

        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> InterpreterResult<Literal> {
        match expr {
            Expr::Empty => Ok(Literal::Nil),
            Expr::Literal(literal) => Ok(literal),
            Expr::Grouping(expr) => self.evaluate(*expr),
            Expr::Unary(operator, right) => {
                let right = self.evaluate(*right);
                match (operator.token_type, right.clone()) {
                    (TokenType::Minus, Ok(Literal::Number(n))) => Ok(Literal::Number(-n)),
                    (TokenType::Minus, _) => Err(RuntimeException::base(
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
            Expr::Assign(name, value) => {
                let expr = Expr::Assign(name.clone(), value.clone());
                let value = self.evaluate(*value)?;
                let distance = self.locals.get(&expr);
                if let Some(distance) = distance {
                    self.environment.assign_at(*distance, name, value.clone())?;
                } else {
                    self.environment.assign(name, value.clone())?;
                }
                Ok(value)
            }
            Expr::Variable(ref name) => self.look_up_variable(name.clone(), expr),
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(*left)?;

                if operator.token_type == TokenType::Or && self.is_truthy(&left) {
                    return Ok(left);
                }

                if !self.is_truthy(&left) {
                    return Ok(left);
                }

                self.evaluate(*right)
            }
            Expr::Lambda(arguments, body) => {
                let stmt = Stmt::Function(Token::from_str(""), arguments, body);
                let function = LoxFunction::new("".to_string(), stmt, self.environment.clone());
                Ok(Literal::LoxFunction(function))
            }
            Expr::Call(callee, paren, arguments) => {
                let callee2 = self.evaluate(*callee.clone())?;
                let mut args = vec![];
                for argument in *arguments {
                    args.push(self.evaluate(argument)?);
                }

                match callee2 {
                    Literal::LoxFunction(mut lf) => {
                        if args.len() != lf.arity() as usize {
                            let message = format!(
                                "Expected {} arguments but got {}.",
                                lf.arity(),
                                args.len()
                            );
                            return Err(RuntimeException::base(paren, message));
                        }
                        let result = lf.call(self, &args);
                        match *callee {
                            Expr::Variable(token) => {
                                self.environment.assign(token, Literal::LoxFunction(lf))?;
                            }
                            _ => (),
                        }
                        result
                    }
                    Literal::NativeFunction(mut nf) => {
                        if args.len() != nf.arity() as usize {
                            let message = format!(
                                "Expected {} arguments but got {}.",
                                nf.arity(),
                                args.len()
                            );
                            return Err(RuntimeException::base(paren, message));
                        }
                        nf.call(self, &args)
                    }
                    _ => {
                        return Err(RuntimeException::base(
                            paren,
                            "Can only call functions and classes.".to_string(),
                        ));
                    }
                }
            }
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(*left);
                let right = self.evaluate(*right);
                match (operator.token_type, left, right) {
                    (TokenType::Minus, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a - b))
                    }
                    (TokenType::Minus, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Slash, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        if b == 0.0 {
                            Err(RuntimeException::base(
                                operator,
                                "Cannot divide by zero".to_string(),
                            ))
                        } else {
                            Ok(Literal::Number(a / b))
                        }
                    }
                    (TokenType::Slash, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Star, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a * b))
                    }
                    (TokenType::Star, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Plus, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a + b))
                    }
                    (TokenType::Plus, Ok(Literal::String(mut s)), Ok(Literal::String(s2))) => {
                        s.push_str(&s2);
                        Ok(Literal::String(s))
                    }
                    (TokenType::Plus, Ok(Literal::String(mut s)), Ok(literal)) => {
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
                        Err(RuntimeException::base(
                            operator,
                            "Operands must be two numbers or two strings.".to_string(),
                        ))
                    }
                    (TokenType::Percent, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::Number(a % b))
                    }
                    (TokenType::Percent, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers".to_string(),
                    )),
                    (TokenType::Greater, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a > b))
                    }
                    (TokenType::Greater, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::GreaterEqual, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a >= b))
                    }
                    (TokenType::GreaterEqual, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::Less, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a < b))
                    }
                    (TokenType::Less, _, _) => Err(RuntimeException::base(
                        operator,
                        "Operands must be numbers.".to_string(),
                    )),
                    (TokenType::LessEqual, Ok(Literal::Number(a)), Ok(Literal::Number(b))) => {
                        Ok(Literal::from(a <= b))
                    }
                    (TokenType::LessEqual, _, _) => Err(RuntimeException::base(
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
        !matches!(v, Literal::Nil | Literal::False)
    }

    fn is_equal(&self, a: &Literal, b: &Literal) -> bool {
        match (a, b) {
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Nil, _) => false,
            (Literal::True, Literal::True) => true,
            (Literal::False, Literal::False) => true,
            (Literal::Number(i), Literal::Number(j)) => i == j,
            (Literal::String(s1), Literal::String(s2)) => s1 == s2,
            (Literal::NativeFunction(f1), Literal::NativeFunction(f2)) => {
                f1.name == f2.name && f1.arity == f2.arity
            }
            _ => false,
        }
    }

    fn interpret(&mut self, stmts: Vec<Stmt>) -> InterpreterResult<()> {
        for stmt in stmts {
            self.execute(stmt)?;
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
            Literal::NativeFunction(_) => "<native fn>".to_string(),
            Literal::LoxFunction(f) => format!("<fn {}>", f.name),
        }
    }

    fn look_up_variable(&self, name: Token, expr: Expr) -> InterpreterResult<Literal> {
        let distance = self.locals.get(&expr);
        if let Some(distance) = distance {
            return self.environment.get_at(*distance, name.lexeme);
        }
        self.environment.get(name)
    }
}
