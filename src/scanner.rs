use std::collections::HashMap;

use crate::token::Token;
use crate::token::TokenType;
use crate::token::Literal;

trait StringFuncs {
    fn substring(&self, start: usize, end: usize) -> &str;
    fn char_at(&self, index: usize) -> char;
}

impl StringFuncs for String {
    fn substring(&self, start: usize, end: usize) -> &str {
        &self[start .. end]
    }

    fn char_at(&self, index: usize) -> char {
        self.bytes().nth(index).unwrap() as char
    }
}

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    pub line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        let keywords: HashMap<String, TokenType> = HashMap::from([
            ("and".to_string(), TokenType::And),
            ("class".to_string(), TokenType::Class),
            ("else".to_string(), TokenType::Else),
            ("false".to_string(), TokenType::False),
            ("for".to_string(), TokenType::For),
            ("fun".to_string(), TokenType::Fun),
            ("if".to_string(), TokenType::If),
            ("nil".to_string(), TokenType::Nil),
            ("or".to_string(), TokenType::Or),
            ("print".to_string(), TokenType::Print),
            ("return".to_string(), TokenType::Return),
            ("super".to_string(), TokenType::Super),
            ("this".to_string(), TokenType::This),
            ("true".to_string(), TokenType::True),
            ("var".to_string(), TokenType::Var),
            ("while".to_string(), TokenType::While)
        ]);
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            keywords
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), std::io::Error> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.line as u32,
        });
        Ok(())
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), std::io::Error> {
        let c = self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen, None);
                Ok(())
            }
            ')' => {
                self.add_token(TokenType::RightParen, None);
                Ok(())
            }
            '{' => {
                self.add_token(TokenType::LeftBrace, None);
                Ok(())
            }
            '}' => {
                self.add_token(TokenType::RightBrace, None);
                Ok(())
            }
            ',' => {
                self.add_token(TokenType::Comma, None);
                Ok(())
            }
            '.' => {
                self.add_token(TokenType::Dot, None);
                Ok(())
            }
            '-' => {
                self.add_token(TokenType::Minus, None);
                Ok(())
            }
            '+' => {
                self.add_token(TokenType::Plus, None);
                Ok(())
            }
            ';' => {
                self.add_token(TokenType::Semicolon, None);
                Ok(())
            }
            '*' => {
                self.add_token(TokenType::Star, None);
                Ok(())
            }
            '!' => {
                let token_type = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(token_type, None);
                Ok(())
            }
            '=' => {
                let token_type = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(token_type, None);
                Ok(())
            }
            '<' => {
                let token_type = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(token_type, None);
                Ok(())
            }
            '>' => {
                let token_type = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token_type, None);
                Ok(())
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() { self.advance(); }
                } else {
                    self.add_token(TokenType::Slash, None);
                }
                Ok(())
            }
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            '"' => {
                self.string()
            }
            'o' => {
                if self.matches('r') {
                    self.add_token(TokenType::Or, None);
                }
                Ok(())
            }

            _ => {
                if c.is_digit(10) {
                    self.number()
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier()
                } else {
                    let message = format!("Unexpected character '{}'", c);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, message))
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let lexeme = &self.source[self.start..self.current];
        let token = Token::new(token_type, lexeme.to_string(), literal, self.line as u32);
        self.tokens.push(token);
    }

    fn current_char(&self) -> char {
        self.source.char_at(self.current)
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        let c = self.current_char();
        if c != expected { return false; }

        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.current_char()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 > self.source.len() { return '\0' }
        self.source.char_at(self.current + 1)
    }

    fn string(&mut self) -> Result<(), std::io::Error> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }

        if self.is_at_end() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Unterminated string."))
        }

        self.advance();

        let value = self.source.substring(self.start + 1, self.current - 1);
        let literal = Literal::String(value.to_string());
        self.add_token(TokenType::String, Some(literal));
        Ok(())
    }

    fn number(&mut self) -> Result<(), std::io::Error> {
        while self.peek().is_digit(10) { self.advance(); }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();

            while self.peek().is_digit(10) { self.advance(); }
        }
        let value = &self.source.substring(self.start, self.current);
        let n: f64 = value.parse::<f64>().unwrap();
        let literal = Literal::Number(n);
        self.add_token(TokenType::Number, Some(literal));
        Ok(())
    }

    fn identifier(&mut self) -> Result<(), std::io::Error> {
        while self.peek().is_ascii_alphanumeric() { self.advance(); }
        let text = self.source.substring(self.start, self.current);
        match self.keywords.get(text) {
            Some(token_type) =>  {
                self.add_token(*token_type, None)
            }
            _ => self.add_token(TokenType::Identifier, None)
        }
        Ok(())
    }
}
