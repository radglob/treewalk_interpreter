use std::error::Error;
use std::fmt;

use crate::token::Token;

#[derive(Debug)]
pub struct ParserError {
    pub token: Token,
    message: String,
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
