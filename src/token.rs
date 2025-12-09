use crate::token_type::*;

pub struct Token {
    pub ttype: TokenType,
    pub lexeme: String,
    pub line: usize,
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
