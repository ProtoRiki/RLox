use crate::token_type::TokenType;
use crate::token_literal::TokenLiteral;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: TokenLiteral,
    pub line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: TokenLiteral, line: i32) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}