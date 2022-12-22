use std::collections::HashMap;

use crate::token::{Literal,Token};
use crate::error::RuntimeError;

#[derive(Clone)]
pub struct Environment {
    enclosing: Box<Option<Environment>>,
    values: HashMap<String, Literal>
}

impl Environment {
    pub fn new() -> Self {
        Self { enclosing: Box::new(None), values: HashMap::new() }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self { enclosing: Box::new(Some(enclosing)), values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), RuntimeError>  {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(())
        }

        if let Some(ref mut env) = *self.enclosing {
            env.assign(name, value)?;
            return Ok(())
        }

        let message = format!("Undefined variable {}.", name.lexeme);
        Err(RuntimeError::new(name, message))
    }

    pub fn get(&self, name: Token) -> Result<Literal, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                match &*self.enclosing {
                    Some(env) => env.get(name),
                    _ => {
                        let message = format!("Undefined variable {}.", name.lexeme);
                        Err(RuntimeError::new(name, message))
                    }
                }
            }
        }
    }
}
