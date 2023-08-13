use std::mem;
use std::rc::Rc;

use crate::expression::Expr::{self, *};
use crate::lox;
use crate::statement::Stmt;
use crate::function_object::FunctionObject;
use crate::token::Token;
use crate::token_literal::TokenLiteral;
use crate::token_type::TokenType::{self, *};

const FUNCTION_ARGUMENT_LIMIT: usize = 255;

pub struct Parser {
    tokens: Vec<Token>,
    current: i32,
    curr_id: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, curr_id: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let line_statement = self.declaration()?;
            statements.push(line_statement);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[FUN]) {
            return self.function_declaration(String::from("function"))
        }

        if self.match_token(&[VAR]) {
            return self.var_declaration();
        }
        self.statement().map_err(|error| { self.synchronize(); error })
    }

    fn function_declaration(&mut self, function_type: String) -> Result<Stmt, String> {
        let name = self.consume(IDENTIFIER, &format!("Expect {function_type} name"))?;
        self.consume(LEFT_PAREN, &format!("Expect '(' after {function_type} name"))?;
        let mut parameters = Vec::new();
        if !self.check(RIGHT_PAREN) {
            if parameters.len() >= FUNCTION_ARGUMENT_LIMIT {
                lox::error(self.peek().line, &format!("Can't have more than {FUNCTION_ARGUMENT_LIMIT} parameters."));
            }
            parameters.push(self.consume(IDENTIFIER, "Expect parameter name.")?);

            while self.match_token(&[COMMA]) {
                if parameters.len() >= FUNCTION_ARGUMENT_LIMIT {
                    lox::error(self.peek().line, &format!("Can't have more than {FUNCTION_ARGUMENT_LIMIT} parameters."));
                }
                parameters.push(self.consume(IDENTIFIER, "Expect parameter name.")?);
            }
        }
        self.consume(RIGHT_PAREN, "Expect ')' after parameters.")?;
        self.consume(LEFT_BRACE, &format!("Expect '{{' before {function_type} body"))?;
        let body = self.block_statement()?;
        Ok(Stmt::Function { ptr: Rc::new(FunctionObject {name, params: parameters, body })})
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume(IDENTIFIER, "Expect variable name.")?;
        let mut initializer = Box::new(Literal { value: TokenLiteral::LOX_NULL });
        if self.match_token(&[EQUAL]) {
            initializer = self.expression()?;
        }
        self.consume(SEMICOLON, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[PRINT]) {
            return self.print_statement();
        }

        if self.match_token(&[LEFT_BRACE]) {
            let statements = self.block_statement()?;
            return Ok(Stmt::Block {statements});
        }

        if self.match_token(&[IF]) {
            return self.if_statement();
        }

        if self.match_token(&[WHILE]) {
            return self.while_statement();
        }

        if self.match_token(&[FOR]) {
            return self.for_statement();
        }

        if self.match_token(&[RETURN]) {
            return self.return_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after value")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(SEMICOLON, "Expect ';' after expression")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn block_statement(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.check(RIGHT_BRACE) && !self.is_at_end() {
            let declaration = self.declaration()?;
            statements.push(declaration)
        }
        self.consume(RIGHT_BRACE, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LEFT_PAREN, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after if-condition")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[ELSE]) {
            Box::new(self.statement()?)
        } else {
            Box::new(Stmt::Expression { expression: Box::new(Literal { value: TokenLiteral::LOX_NULL })})
        };
        Ok(Stmt::If {expression: condition, then_branch, else_branch})
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LEFT_PAREN, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(RIGHT_PAREN, "Expect ')' after while-condition")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While {expression: condition, body})
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(LEFT_PAREN, "Expect '(' after 'for'")?;
        // ; -> initializer omitted
        // var -> initializer included
        // no var -> no initialization, must be expression
        let (initializer, had_initializer) = match (self.match_token(&[SEMICOLON]), self.match_token(&[VAR])) {
            (true, _) => (Stmt::Expression { expression: Box::new(Literal { value: TokenLiteral::LOX_NULL })}, false),
            (false, true) => (self.var_declaration()?, true),
            (false, false) => (self.expression_statement()?, true),
        };

        let condition = if !self.check(SEMICOLON) {
            self.expression()?
        } else {
            Box::new(Literal { value: TokenLiteral::LOX_BOOL(true) })
        };
        self.consume(SEMICOLON, "Expect ';' after loop condition")?;

        let (increment, had_increment) = if !self.check(RIGHT_PAREN) {
            (self.expression()?, true)
        } else {
            (Box::new(Literal { value: TokenLiteral::LOX_NULL }), false)
        };
        self.consume(RIGHT_PAREN, "Expect ')' after for clause")?;

        let body = self.statement()?;

        // De-sugar the for-loop into a while-loop

        let mut statements = Vec::new();

        if had_initializer {
            statements.push(initializer);
        }

        let mut body = match body {
            Stmt::Block { statements } => statements,
            _ => vec![body]
        };

        if had_increment {
            body.push(Stmt::Expression {expression: increment});
        }

        let body = Box::new(Stmt::Block { statements: body});
        if statements.is_empty() {
            Ok(Stmt::While { expression: condition, body})
        } else {
            statements.push(Stmt::While { expression: condition, body});
            Ok(Stmt::Block {statements})
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.take_previous();
        let value = if !self.check(SEMICOLON) { self.expression()? } else {
            Box::new(Literal { value: TokenLiteral::LOX_NULL })
        };
        self.consume(SEMICOLON, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn expression(&mut self) -> Result<Box<Expr>, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<Expr>, String> {
        let expr = self.or()?;
        if self.match_token(&[EQUAL]) {
            let equals = self.take_previous();
            // Assignment is right-associative, recursively call assignment to parse rhs
            let value = self.assignment()?;
            return match *expr {
                // Convert the r-value expression node into an l-value representation.
                Variable { name , .. } => {
                    let id = self.curr_id;
                    self.curr_id += 1;
                    Ok(Box::new(Assign { name, value, id }))
                },
                _ => {
                    // Error if left-hand-side is an invalid assignment target
                    // Report error but do not throw it
                    lox::token_error(&equals, "Invalid assignment target.");
                    Ok(expr)
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.and()?;
        while self.match_token(&[OR]) {
            let operator = self.take_previous();
            let right = self.and()?;
            expr = Box::new(Logical { left: expr, operator, right });
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.equality()?;
        while self.match_token(&[AND]) {
            let operator = self.take_previous();
            let right = self.equality()?;
            expr = Box::new(Logical {left: expr, operator, right });
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.comparison()?;
        while self.match_token(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.take_previous();
            let right = self.comparison()?;
            left = Box::new(Binary { left, operator, right, });
        }
        Ok(left)
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types.iter() {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
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

    fn take_previous(&mut self) -> Token {
        let dest = &mut self.tokens[(self.current - 1) as usize];
        mem::replace(dest, Token::new(NIL, String::new(), TokenLiteral::LOX_NULL, -1))
    }

    fn comparison(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.term()?;
        while self.match_token(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.take_previous();
            let right = self.term()?;
            left = Box::new(Binary { left, operator, right, });
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.factor()?;
        while self.match_token(&[MINUS, PLUS]) {
            let operator = self.take_previous();
            let right = self.factor()?;
            left = Box::new(Binary { left, operator, right, })
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Box<Expr>, String> {
        let mut left = self.unary()?;
        while self.match_token(&[SLASH, STAR]) {
            let operator = self.take_previous();
            let right = self.unary()?;
            left = Box::new(Binary { left, operator, right, });
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Box<Expr>, String> {
        if self.match_token(&[BANG, MINUS]) {
            let operator = self.take_previous();
            let right = self.unary()?;
            return Ok(Box::new(Unary { operator, right }));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[LEFT_PAREN]) {
                expr = self.finish_call(expr)?;
            }
            else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> Result<Box<Expr>, String> {
        let mut arguments = Vec::new();
        if !self.check(RIGHT_PAREN) {
            // Parse each argument as an expression
            arguments.push(*self.expression()?);
            // Look for a comma after every expression
            while self.match_token(&[COMMA]) {
                if arguments.len() >= FUNCTION_ARGUMENT_LIMIT {
                    lox::token_error(self.peek(), &format!("Can't have more than {FUNCTION_ARGUMENT_LIMIT} arguments."));
                }
                arguments.push(*self.expression()?);
            }
        }
        let paren = self.consume(RIGHT_PAREN, "Expect ')' after arguments.")?;
        Ok(Box::new(Call { callee, paren, arguments }))
    }

    fn primary(&mut self) -> Result<Box<Expr>, String> {
        if self.match_token(&[NUMBER, STRING]) {
            return Ok(Box::new(Literal { value: self.take_previous().literal }));
        }

        if self.match_token(&[TRUE]) {
            return Ok(Box::new(Literal { value: TokenLiteral::LOX_BOOL(true) }));
        }

        if self.match_token(&[FALSE]) {
            return Ok(Box::new(Literal { value: TokenLiteral::LOX_BOOL(false) }));
        }

        if self.match_token(&[NIL]) {
            return Ok(Box::new(Literal { value: TokenLiteral::LOX_NULL }));
        }

        if self.match_token(&[IDENTIFIER]) {
            let id = self.curr_id;
            self.curr_id += 1;
            return Ok(Box::new(Variable { name: self.take_previous(), id }));
        }

        if self.match_token(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Box::new(Grouping { expression: expr }));
        }

        let err_msg = String::from("Expected expression");
        lox::token_error(self.peek(), &err_msg);
        Err(err_msg)
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            self.advance();
            return Ok(self.take_previous());
        }
        lox::token_error(self.peek(), message);
        Err(String::from(message))
    }

    // Recover when parser panics to move to the beginning of the next declaration
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.take_previous().token_type == SEMICOLON {
                return;
            }
            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => {
                    return;
                }
                _ => (),
            }
        }
        self.advance();
    }
}
