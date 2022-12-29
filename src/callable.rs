use crate::token::Literal;
use crate::interpreter::{Interpreter,InterpreterResult};

pub trait Callable {
    fn arity(&self) -> u8;
    fn call(&self, interpreter: &Interpreter, args: &Vec<Literal>) -> InterpreterResult<Literal>;
}

pub fn as_callable(literal: &Literal) -> Option<&dyn Callable> {
    match literal {
        Literal::NativeFunction(f) => Some(f),
        Literal::LoxFunction(f) => Some(f),
        _ => None,
    }
}


