use std::collections::HashMap;

use crate::token::{Literal,Token};
use crate::error::RuntimeError;

pub struct Environment {
    values: HashMap<String, Literal>
}

impl Environment {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Token) -> Result<Literal, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                let message = format!("Undefined variable {}.", name.lexeme);
                Err(RuntimeError::new(name, message))
            }
        }
    }
}
