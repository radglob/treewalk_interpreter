use crate::callable::Callable;
use crate::environment::Environment;
use crate::error::RuntimeException;
use crate::interpreter::Interpreter;
use crate::stmt::Stmt;
use crate::token::Literal;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub name: String,
    declaration: Box<Stmt>,
    pub closure: Environment,
}

impl LoxFunction {
    pub fn new(name: String, declaration: Stmt, closure: Environment) -> Self {
        Self {
            name,
            declaration: Box::new(declaration),
            closure,
        }
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
        &mut self,
        interpreter: &Interpreter,
        args: &Vec<Literal>,
    ) -> Result<Literal, RuntimeException> {
        let (env, depth) = Environment::wrap(self.closure.clone(), interpreter.environment.clone(), 0);
        let mut interpreter2 = Interpreter::new(&env);
        match &*self.declaration {
            Stmt::Function(_name, params, body) => {
                for (i, param) in params.iter().enumerate() {
                    let value: Literal = args.get(i).unwrap().clone();
                    interpreter2.environment.define(param.lexeme.clone(), value);
                }

                let result = interpreter2.evaluate_block(*(*body).clone());
                self.closure = Environment::unwrap(interpreter2.environment, depth);
                match result {
                    Err(RuntimeException::Return(r)) => match r.value {
                        Some(v) => return Ok(v),
                        None => return Ok(Literal::Nil),
                    },
                    Err(err) => return Err(err),
                    _ => return Ok(Literal::Nil),
                }
            }
            _ => Err(RuntimeException::base(
                Token::default(),
                "Invalid function declaration.".to_string(),
            )),
        }
    }
}
