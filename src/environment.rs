use std::collections::HashMap;

use crate::error::{RuntimeError, RuntimeException};
use crate::token::{Literal, Token};

#[derive(Clone, Debug)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Literal>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self {
            enclosing: Some(Box::new(enclosing)),
            values: HashMap::new(),
        }
    }

    pub fn wrap(env: Environment, enclosing: Environment, depth: u32) -> (Self, u32) {
        match env.enclosing {
            None => (
                Self {
                    enclosing: Some(Box::new(enclosing)),
                    ..env.clone()
                },
                depth,
            ),
            Some(ref enc) => {
                let (e, d) = Environment::wrap(*enc.clone(), enclosing, depth + 1);
                return (
                    Self {
                        enclosing: Some(Box::new(e)),
                        ..env.clone()
                    },
                    d,
                );
            }
        }
    }

    pub fn unwrap(env: Environment, mut depth: u32) -> Self {
        let mut env = env.clone();
        let mut r = &mut env;

        while depth > 0 {
            match r.enclosing {
                None => panic!(),
                Some(ref mut enc) => {
                    r = enc;
                }
            }
            depth -= 1;
        }

        r.enclosing = None;
        env
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: Literal) -> Result<(), RuntimeException> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme, value);
            return Ok(());
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
            None => match &self.enclosing {
                Some(env) => (*env).get(name),
                _ => {
                    let message = format!("Undefined variable {}.", name.lexeme);
                    Err(RuntimeException::base(name, message))
                }
            },
        }
    }
}
