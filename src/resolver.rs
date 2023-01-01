use std::collections::HashMap;

use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Token;

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

trait Resolve<T> {
    fn resolve(&mut self, value: T);
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        };
        let mut scope = self.scopes.pop().expect("Expected a HashMap.");
        scope.insert(name.lexeme, false);
        self.scopes.push(scope);
    }

    fn define(&mut self, name: Token) {
        if self.scopes.is_empty() {
            return;
        }
        let mut scope = self.scopes.pop().expect("Expected a HashMap.");
        scope.insert(name.lexeme, true);
        self.scopes.push(scope);
    }

    fn resolve_local(&self, expr: Expr, name: Token) {
        let mut i = self.scopes.len() - 1;
        loop {
            if self.scopes.get(i).unwrap().contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expr.clone(), (self.scopes.len() - 1 - i) as u32);
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    fn resolve_function(&mut self, params: Vec<Token>, body: Box<Vec<Stmt>>) {
        self.begin_scope();
        for param in params {
            self.declare(param.clone());
            self.define(param);
        }
        self.resolve(*body);
        self.end_scope();
    }
}

impl Resolve<Vec<Stmt>> for Resolver {
    fn resolve(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            self.resolve(stmt);
        }
    }
}

impl Resolve<Stmt> for Resolver {
    fn resolve(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts);
                self.end_scope();
            }
            Stmt::Var(name, initializer) => {
                self.declare(name.clone());
                if let Some(expr) = initializer {
                    self.resolve(expr)
                }
                self.define(name);
            }
            Stmt::Function(name, params, body) => {
                self.declare(name.clone());
                self.define(name);
                self.resolve_function(params, body);
            }
            Stmt::Expression(expression) => {
                self.resolve(expression);
            }
            Stmt::If(condition, then_branch, else_branch) => {
                let (then_branch, else_branch) = (*then_branch, *else_branch);
                self.resolve(condition);
                self.resolve(then_branch);
                if else_branch.is_some() {
                    self.resolve(else_branch.unwrap());
                }
            }
            Stmt::Print(expression) => {
                self.resolve(expression);
            }
            Stmt::Return(_, value) => {
                let value = *value;
                if value.is_some() {
                    self.resolve(value.unwrap());
                }
            }
            Stmt::While(condition, body) => {
                self.resolve(condition);
                self.resolve(*body);
            }
            Stmt::Break(_) => (),
        }
    }
}

impl Resolve<Expr> for Resolver {
    fn resolve(&mut self, expr: Expr) {
        match expr {
            Expr::Variable(ref name) => {
                if !self.scopes.is_empty() {
                    let scope = self.scopes.last().unwrap();
                    match scope.get(&name.lexeme) {
                        Some(false) => {
                            self.interpreter.log_error(
                                name.clone(),
                                "Can't read local variable in its own initializer.".to_string(),
                            ).expect("There was an error printing to stderr.");
                        }
                        _ => (),
                    }
                }
                self.resolve_local(expr.clone(), name.clone())
            }
            Expr::Assign(name, value) => {
                let expr = Expr::Assign(name.clone(), value.clone());
                self.resolve(*value);
                self.resolve_local(expr, name);
            }
            Expr::Binary(left, _, right) => {
                self.resolve(*left);
                self.resolve(*right);
            }
            Expr::Call(callee, _, arguments) => {
                self.resolve(*callee);
                for argument in *arguments {
                    self.resolve(argument);
                }
            }
            Expr::Grouping(expression) => {
                self.resolve(*expression);
            }
            Expr::Literal(_) => (),
            Expr::Logical(left, _, right) => {
                self.resolve(*left);
                self.resolve(*right);
            }
            Expr::Unary(_, right) => {
                self.resolve(*right);
            }
            _ => panic!(),
        }
    }
}
