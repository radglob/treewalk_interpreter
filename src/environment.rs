use std::collections::HashMap;

use crate::token::{Literal,Token};
use crate::error::{RuntimeError,RuntimeException};

#[derive(Clone,Debug)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self { enclosing: None, values: HashMap::new() }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self { enclosing: Some(Box::new(enclosing)), values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), RuntimeException>  {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(())
        }

        match &mut self.enclosing {
            Some(enclosing) => enclosing.assign(name, value),
            None => {
                let message = format!("Undefined variable {}.", name.lexeme);
                Err(RuntimeException::Base(RuntimeError::new(name, message)))
            }
        }
    }

    pub fn get(&self, name: Token) -> Result<Literal, RuntimeException> {
        match self.values.get(&name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                match &self.enclosing {
                    Some(env) => (*env).get(name),
                    _ => {
                        let message = format!("Undefined variable {}.", name.lexeme);
                        Err(RuntimeException::base(name, message))
                    }
                }
            }
        }
    }
}
