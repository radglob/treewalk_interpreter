use crate::expr::Expr;
use crate::token::Literal;

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, expr: Expr) -> String {
        self.output(expr)
    }

    fn output(&self, expr: Expr) -> String {
        match expr {
            Expr::Literal(literal) => self.parenthesize_literal(literal),
            Expr::Unary(operator, right) => self.parenthesize(operator.lexeme, vec![*right]),
            Expr::Binary(left, operator, right) => {
                self.parenthesize(operator.lexeme, vec![*left, *right])
            }
            Expr::Grouping(expr) => self.parenthesize("group".to_string(), vec![*expr]),
            Expr::Variable(token) => format!("(var {})", token.lexeme)
        }
    }

    fn parenthesize(&self, name: String, exprs: Vec<Expr>) -> String {
        let mut s = String::from("(");
        s.push_str(&name);
        for expr in exprs {
            s.push(' ');
            s.push_str(&self.output(expr));
        }
        s.push(')');
        s
    }

    fn parenthesize_literal(&self, literal: Literal) -> String {
        match literal {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => s,
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
            Literal::Nil => "nil".to_string(),
        }
    }
}
