use crate::lox;

use crate::token::Token;
use crate::expression::Expr::{self, *};
use crate::token_type::TokenType::{self, *};
use crate::statement::Stmt;

use std::mem;
use crate::token_literal::TokenLiteral::{LOX_BOOL, NULL};

pub struct Parser {
    tokens: Vec<Token>,
    current: i32,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let line_statement = self.statement()?;
            statements.push(line_statement);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[PRINT]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Box<Expr>, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, String>{
        let mut left = self.comparison()?;
        while self.match_token(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.previous();
            let right = self.comparison()?;
            left = Box::from(Binary { left, operator, right });
        }
        Ok(left)
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current as usize]
    }

    fn previous(&mut self) -> Token {
        let dest = &mut self.tokens[(self.current - 1) as usize];
        mem::replace(dest, Token::new(NIL, String::new(), NULL, -1))
    }

    fn comparison(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.term()?;
        while self.match_token(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous();
            let right = self.term()?;
            left = Box::from(Binary { left, operator, right });
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.factor()?;
        while self.match_token(&[MINUS, PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            left = Box::from(Binary { left, operator, right} )
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.unary()?;
        while self.match_token(&[SLASH, STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            left = Box::from(Binary { left, operator, right });
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Box<Expr>, String> {
        if self.match_token(&[BANG, MINUS]) {
            let operator = self.previous();
            return match self.unary() {
                Ok(right) => Ok(Box::from(Unary { operator, right })),
                Err(msg) => Err(msg)
            };
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Box<Expr>, String> {
        if self.match_token(&[NUMBER, STRING]) {
            return Ok(Box::new(Literal {value: self.previous().literal}));
        }
        if self.match_token(&[TRUE]) {
            return Ok(Box::new(Literal {value: LOX_BOOL(true)}));
        }
        if self.match_token(&[FALSE]) {
            return Ok(Box::new(Literal {value: LOX_BOOL(false)}));
        }
        if self.match_token(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping { expression: expr }));
        }
        lox::token_error(self.peek(), "Expect expression.");
        Err(String::from("Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, String>{
        if self.check(&token_type) {
            self.advance();
            return Ok(self.previous())
        }
        lox::token_error(self.peek(), message);
        Err(String::from(message))
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == SEMICOLON {
                return;
            }
            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => {
                    return;
                }
                _ => ()
            }
        }
        self.advance();
    }

}