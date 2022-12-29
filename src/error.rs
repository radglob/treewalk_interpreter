use std::error::Error;
use std::fmt;

use crate::token::Token;
use crate::token::Literal;

#[derive(Debug)]
pub struct ParserError {
    pub token: Token,
    pub message: String,
}

impl ParserError {
    pub fn new(token: Token, message: std::string::String) -> Self {
        Self { token, message }
    }
}

impl Error for ParserError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug,Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String
}

impl RuntimeError {
    pub fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Clone,Debug)]
pub struct Return {
    pub value: Option<Literal>
}

impl Return {
    pub fn new(value: Option<Literal>) -> Self {
        Self { value }
    }
}

#[derive(Clone,Debug)]
pub enum RuntimeException {
    Base(RuntimeError),
    Return(Return),
    Break
}

impl RuntimeException {
    pub fn base(token: Token, message: String) -> Self {
        let runtime_error = RuntimeError::new(token, message);
        RuntimeException::Base(runtime_error)
    }

    pub fn r#return(value: Option<Literal>) -> Self {
        let r = Return::new(value);
        RuntimeException::Return(r)
    }
}
