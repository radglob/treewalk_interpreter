use crate::expr::Expr;
use crate::token::Literal;

pub struct AstPrinter;

impl Default for AstPrinter {
    fn default() -> Self {
        Self::new()
    }
}

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
            Expr::Variable(token) => format!("(var {})", token.lexeme),
            Expr::Assign(token, value) => format!("(var {} {})", token.lexeme, self.output(*value)),
            Expr::Logical(left, operator, right) => format!(
                "({} {} {})",
                operator.lexeme,
                self.output(*left),
                self.output(*right)
            ),
            Expr::Call(callee, _, arguments) => {
                let mut s = self.output(*callee);
                for arg in *arguments {
                    s.push_str(&self.output(arg));
                    s.push(' ');
                }
                s.push(')');
                s
            }
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
        literal.to_string()
    }
}
