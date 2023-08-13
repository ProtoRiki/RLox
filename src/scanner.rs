use std::mem;
use std::str::{self, FromStr};
use std::rc::Rc;

use crate::lox;
use crate::token::Token;
use crate::token_literal::TokenLiteral::{self, *};
use crate::token_type::TokenType::{self, *};

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: i32,
    current: i32,
    line: i32,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(EOF, String::from(""), LOX_NULL, self.line));
        mem::take(&mut self.tokens)
    }

    /// None gets the current char
    /// Otherwise, use the passed index
    fn get_source_char(&self, index: Option<usize>) -> u8 {
        match index {
            None => self.source.as_bytes()[self.current as usize],
            Some(i) => {
                if i >= self.source.len() {
                    return b'\0'
                }
                self.source.as_bytes()[i]
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current as usize >= self.source.len()
    }

    fn scan_token(&mut self) {
        match self.advance() {
            // Operators
            b'(' => self.add_token_nonliteral(LEFT_PAREN),
            b')' => self.add_token_nonliteral(RIGHT_PAREN),
            b'{' => self.add_token_nonliteral(LEFT_BRACE),
            b'}' => self.add_token_nonliteral(RIGHT_BRACE),
            b',' => self.add_token_nonliteral(COMMA),
            b'.' => self.add_token_nonliteral(DOT),
            b'-' => self.add_token_nonliteral(MINUS),
            b'+' => self.add_token_nonliteral(PLUS),
            b';' => self.add_token_nonliteral( SEMICOLON),
            b'*' => self.add_token_nonliteral(STAR),
            b'!' => {
                match self.match_second(b'=') {
                    true => self.add_token_nonliteral(BANG_EQUAL),
                    false => self.add_token_nonliteral(BANG)
                }
            },
            b'=' => {
                match self.match_second(b'=') {
                    true => self.add_token_nonliteral(EQUAL_EQUAL),
                    false => self.add_token_nonliteral(EQUAL)
                }
            }
            b'<' => {
                match self.match_second(b'=') {
                    true => self.add_token_nonliteral(LESS_EQUAL),
                    false => self.add_token_nonliteral(LESS)
                }
            }
            b'>' => {
                match self.match_second(b'=') {
                    true => self.add_token_nonliteral(GREATER_EQUAL),
                    false => self.add_token_nonliteral(GREATER)
                }
            }
            b'/' => {
                match self.match_second(b'/') {
                    true => {
                        // Comment goes until end of line
                        while !self.is_at_end() && self.get_source_char(None) != b'\n' {
                            self.advance();
                        }
                    }
                    false => self.add_token_nonliteral(SLASH)
                }
            }

            // Skip whitespace
            b' ' | b'\r' | b'\t' => (),
            b'\n' => self.line += 1,

            // Literals
            b'"' => self.string(),
            b'0'..=b'9' => self.number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),

            // End of file
            b'\0' => (),

            _ => lox::error(self.line, "Unexpected character."),
        }
    }

    fn match_second(&mut self, expected: u8) -> bool {
        if self.is_at_end() { return false }
        if self.get_source_char(None) != expected { return false }

        self.current += 1;
        true
    }

    fn advance(&mut self) -> u8 {
        let i = self.current;
        self.current += 1;
        self.get_source_char(Some(i as usize))
    }

    fn add_token_nonliteral(&mut self, token_type: TokenType) {
        self.add_token(token_type, LOX_NULL);
    }

    fn add_token(&mut self, token_type: TokenType, literal: TokenLiteral) {
        let bytes = self.source.as_bytes();
        let text = String::from_utf8_lossy(&bytes[self.start as usize..self.current as usize]);
        let text = text.into_owned();
        let token = Token::new(token_type, text, literal, self.line);
        self.tokens.push(token);
    }

    fn string(&mut self) {
        while self.get_source_char(None) != b'"' && !self.is_at_end() {
            if self.get_source_char(None) == b'\n' {
                self.line += 1
            }
            self.advance();
        }
        if self.is_at_end() {
            lox::error(self.line, "Unterminated string.");
            return;
        }

        // The closing "."
        self.advance();
        let bytes = self.source.as_bytes();

        // Strip quotes
        let value = String::from_utf8_lossy(&bytes[(self.start+1) as usize..(self.current-1) as usize]);
        let value = Rc::new(value.into_owned());
        self.add_token(STRING, LOX_STRING(value));
    }

    fn is_digit(c: u8) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.get_source_char(None)) {
            self.advance();
        }

        if self.get_source_char(None) == b'.'
            && Scanner::is_digit(self.get_source_char(Some((self.current + 1) as usize))) {
            self.advance();
            while Scanner::is_digit(self.get_source_char(None)) {
                self.advance();
            }
        }
        let bytes = self.source.as_bytes();
        let value = str::from_utf8(&bytes[self.start as usize..self.current as usize]).unwrap();
        let value = f64::from_str(value).unwrap();
        self.add_token(NUMBER, LOX_NUMBER(value));
    }

    fn is_alpha(c: u8) -> bool {
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == b'_'
    }

    fn is_alphanumeric(c: u8) -> bool {
        Scanner::is_digit(c) || Scanner::is_alpha(c)
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.get_source_char(None)) {
            self.advance();
        }
        let bytes = self.source.as_bytes();
        let value = str::from_utf8(&bytes[self.start as usize..self.current as usize]).unwrap();
        let token_type = match value {
            "and" => AND,
            "class" => CLASS,
            "else" => ELSE,
            "false" => FALSE,
            "for" => FOR,
            "fun" => FUN,
            "if" => IF,
            "nil" => NIL,
            "or" => OR,
            "print" => PRINT,
            "return" => RETURN,
            "super" => SUPER,
            "this" => THIS,
            "true" => TRUE,
            "var" => VAR,
            "while" => WHILE,
            _ => IDENTIFIER,
        };
        self.add_token_nonliteral(token_type);
    }
}