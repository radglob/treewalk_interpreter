use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Percent,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Identifiers
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Break,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

impl From<bool> for Literal {
    fn from(v: bool) -> Self {
        if v {
            Self::True
        } else {
            Self::False
        }
    }
}

impl From<f64> for Literal {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<String> for Literal {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::Nil => "nil".to_string(),
            Literal::True => "true".to_string(),
            Literal::False => "false".to_string(),
            Literal::String(s) => s.to_string(),
            Literal::Number(n) => n.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}
