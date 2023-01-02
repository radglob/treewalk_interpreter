use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::callable::Callable;
use crate::error::RuntimeException;
use crate::token::Literal;
use crate::token::Token;
use crate::interpreter::Interpreter;

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: u8,
    pub callable: fn(interpreter: &Interpreter, args: &Vec<Literal>) -> Result<Literal, RuntimeException>,
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NativeFunction({})", self.name)
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(&mut self, interpreter: &Interpreter, args: &Vec<Literal>) -> Result<Literal, RuntimeException> {
        (self.callable)(interpreter, args)
    }
}

pub fn clock(_interpreter: &Interpreter, args: &Vec<Literal>) -> Result<Literal, RuntimeException> {
    if args.len() != 0 {
        let message = format!("Expected 0 args, received {}.", args.len());
        return Err(RuntimeException::base(Token::default(), message))
    }

    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();

    Ok(Literal::Number(since_epoch.as_millis() as f64))
}

