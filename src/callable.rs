use crate::error::RuntimeError;
use crate::token::Literal;

pub trait Callable {
    fn arity(&self) -> u8;
    fn call(&self, args: &Vec<Literal>) -> Result<Literal, RuntimeError>;
}

pub fn as_callable(literal: &Literal) -> Option<&dyn Callable> {
    match literal {
        Literal::NativeFunction(f) => Some(f),
        _ => None,
    }
}


