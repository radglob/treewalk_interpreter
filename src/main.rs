use std::env;
use std::process::exit;
use std::error::Error;

pub mod ast_printer;
pub mod expr;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod token;
pub mod error;

use crate::interpreter::Interpreter;

fn main() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::default();
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 {
        println!("Usage: rlox [script]");
        exit(64);
    } else if args.len() == 1 {
        interpreter.run_file(&args[0])?
    } else {
        interpreter.run_prompt()?
    }
    Ok(())
}
