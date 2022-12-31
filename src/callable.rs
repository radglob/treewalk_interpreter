use crate::interpreter::{Interpreter, InterpreterResult};
use crate::token::Literal;

pub trait Callable {
    fn arity(&self) -> u8;
    fn call(
        &mut self,
        interpreter: &Interpreter,
        args: &Vec<Literal>,
    ) -> InterpreterResult<Literal>;
}
