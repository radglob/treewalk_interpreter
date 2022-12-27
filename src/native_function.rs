use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::callable::Callable;
use crate::error::RuntimeError;
use crate::token::Literal;
use crate::token::Token;

#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: u8,
    pub callable: fn(&Vec<Literal>) -> Result<Literal, RuntimeError>,
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

    fn call(&self, args: &Vec<Literal>) -> Result<Literal, RuntimeError> {
        (self.callable)(args)
    }
}

pub fn clock(args: &Vec<Literal>) -> Result<Literal, RuntimeError> {
    if args.len() != 0 {
        let message = format!("Expected 0 args, received {}.", args.len());
        return Err(RuntimeError::new(Token::default(), message))
    }

    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();

    Ok(Literal::Number(since_epoch.as_millis() as f64))
}

