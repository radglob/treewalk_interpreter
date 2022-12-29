use crate::callable::Callable;
use crate::error::{RuntimeException,RuntimeError};
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Literal;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub name: String,
    declaration: Box<Stmt>,
}

impl LoxFunction {
    pub fn new(name: String, declaration: Stmt) -> Self {
        Self { name, declaration: Box::new(declaration) }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> u8 {
        match &*self.declaration {
            Stmt::Function(_name, params, _body) => params.len() as u8,
            _ => 0,
        }
    }

    fn call(
        &self,
        interpreter: &Interpreter,
        args: &Vec<Literal>,
    ) -> Result<Literal, RuntimeException> {
        let mut interpreter2 = Interpreter::new(&interpreter.environment);
        match &*self.declaration {
            Stmt::Function(_name, params, body) => {
                for (i, param) in params.iter().enumerate() {
                    let value: Literal = args.get(i).unwrap().clone();
                    interpreter2.environment.define(param.lexeme.clone(), value);
                }

                match interpreter2.evaluate_block(*(*body).clone()) {
                    Err(RuntimeException::Return(r)) => {
                        match r.value {
                            Some(v) => return Ok(v),
                            None => return Ok(Literal::Nil)
                        }
                    }
                    Err(err) => return Err(err),
                    _ => return Ok(Literal::Nil)
                }
            }
            _ => Err(RuntimeException::Base(RuntimeError::new(
                Token::default(),
                "Invalid function declaration.".to_string(),
            ))),
        }
    }
}
