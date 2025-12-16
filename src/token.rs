use crate::token_type::*;

#[derive(Debug, PartialEq, Eq, Hash, )]
pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(lexeme:&str) -> Self {
        Self { 
            ttype: TokenType::Undefined, 
            lexeme: lexeme.to_string(), 
            line: 0
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TokenType::Undefined,
            lexeme: String::new(),
            line: 0,
        }
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self {
            ttype: self.ttype,
            lexeme: self.lexeme.clone(),
            line: self.line,
        }
    }
}
