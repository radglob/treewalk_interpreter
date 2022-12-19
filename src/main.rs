use std::env;
use std::fs;
use std::io::{stderr, Write};
use std::process::exit;

pub mod scanner;
pub mod token;

use crate::scanner::Scanner;

pub struct Interpreter {
    had_error: bool,
}

impl Interpreter {
    fn default() -> Self {
        Self { had_error: false }
    }

    fn run_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let contents = fs::read_to_string(path)?;
        self.run(&contents)?;
        if self.had_error {
            exit(65)
        }
        Ok(())
    }

    fn run(&mut self, source: &str) -> Result<(), std::io::Error> {
        let mut scanner = Scanner::new(source.to_string());
        if let Err(_) = scanner.scan_tokens() {
            self.error(scanner.line as u32, "Unexpected character.".to_string())?;
        }

        for token in scanner.tokens {
            println!("{:?}", token);
        }

        Ok(())
    }

    fn run_prompt(&mut self) -> Result<(), std::io::Error> {
        loop {
            let mut input = String::new();
            print!("> ");
            let _ = std::io::stdout().flush();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    self.run(&input)?;
                    self.had_error = false;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn error(&mut self, line: u32, message: String) -> Result<(), std::io::Error> {
        self.report(line, "".to_string(), message)?;
        Ok(())
    }

    fn report(
        &mut self,
        line: u32,
        location: String,
        message: String,
    ) -> Result<(), std::io::Error> {
        writeln!(stderr(), "[line {}] Error{}: {}", line, location, message)?;
        self.had_error = true;
        Ok(())
    }
}

fn main() -> Result<(), std::io::Error> {
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
