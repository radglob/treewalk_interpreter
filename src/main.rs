use std::env;
use std::process::exit;
use std::error::Error;
use std::cmp::Ordering::*;

pub mod ast_printer;
pub mod callable;
pub mod declaration;
pub mod environment;
pub mod error;
pub mod expr;
pub mod interpreter;
pub mod lox_function;
pub mod native_function;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod stmt;
pub mod token;

use crate::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::default();
    let args: Vec<String> = env::args().skip(1).collect();
    match args.len().cmp(&1) {
        Greater => {
            println!("Usage: rlox [script]");
            exit(64);
        }
        Equal => interpreter.run_file(&args[0])?,
        _ => interpreter.run_prompt()?
    }
    Ok(())
}
