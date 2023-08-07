use crate::expression::Expr::{self, *};
use crate::token_literal::TokenLiteral;
use crate::token::Token;
use crate::token_type::TokenType::*;
use crate::lox;
use crate::statement::Stmt::{self, *};

pub struct Interpreter;

pub enum InterpreterError {
    LiteralError(String),
    OperatorError {
        operator: Token,
        msg: String,
    }
}

impl Interpreter {
    pub fn interpret(&self, expr: Box<Expr>) {
        let value = Interpreter::accept_expr(expr);
        match value {
            Ok(literal) => println!("{}", literal.to_string()),
            Err(error) => lox::runtime_error(&error),
        }
    }

    fn accept_expr(expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Binary { left, operator, right } => {
                Interpreter::visit_binary_expr(Box::from(Binary {left, operator, right}))
            },

            Grouping { expression} => {
                Interpreter::visit_grouping_expr(Box::from(Grouping {expression}))
            },

            Literal { value } => {
                Interpreter::visit_literal_expr(Box::from(Literal { value }))
            },

            Unary { operator, right } => {
                Interpreter::visit_unary_expr(Box::from(Unary { operator, right}))
            }
        }
    }

    fn accept_statement(stmt: Stmt) -> Result<(), InterpreterError>{
        match stmt {
            Expression(expr) => Interpreter::visit_expression_stmt(Expression(expr)),
            Print(expr) => Interpreter::visit_print_stmt(Print(expr)),
        }
    }

    fn visit_expression_stmt(stmt: Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Expression(expr) => Interpreter::accept_expr(expr)?,
            _ => {
                let msg = String::from("Non-expression statement passed to expr visitor");
                return Err(InterpreterError::LiteralError(msg))
            },
        };
        Ok(())
    }

    fn visit_print_stmt(stmt: Stmt) -> Result<(), InterpreterError>{
        match stmt {
            Print(expr) => {
                let value = Interpreter::accept_expr(expr)?;
                println!("{value}");
            },
            _ => {
                let msg = String::from("Non-print statement passed to print visitor");
                return Err(InterpreterError::LiteralError(msg))
            },
        }
        Ok(())
    }

    fn visit_literal_expr(expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Literal { value } => Ok(value),
            _ => Err(InterpreterError::LiteralError(String::from("Non-literal expression passed to literal visitor"))),
        }
    }

    fn visit_grouping_expr(expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Grouping { expression } => Interpreter::accept_expr(expression),
            _ => Err(InterpreterError::LiteralError(String::from("Non-group expression passed to group visitor"))),
        }
    }
    fn visit_binary_expr(expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Binary { left, operator, right } => {
                // Recursively evaluate operands until they are usable literals
                let left = Interpreter::accept_expr(left)?;
                let right = Interpreter::accept_expr(right)?;
                match (left, right) {
                    // Two numbers
                    (TokenLiteral::LOX_NUMBER(left), TokenLiteral::LOX_NUMBER(right)) => {
                        match operator.token_type {
                            // Arithmetic
                            PLUS => Ok(TokenLiteral::LOX_NUMBER(left + right)),
                            MINUS => Ok(TokenLiteral::LOX_NUMBER(left - right)),
                            STAR => Ok(TokenLiteral::LOX_NUMBER(left * right)),
                            SLASH => Ok(TokenLiteral::LOX_NUMBER(left / right)),
                            // Logical
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_NUMBER(left);
                                let right = TokenLiteral::LOX_NUMBER(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            },
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_NUMBER(left);
                                let right = TokenLiteral::LOX_NUMBER(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            },
                            GREATER => Ok(TokenLiteral::LOX_BOOL(left > right)),
                            GREATER_EQUAL => Ok(TokenLiteral::LOX_BOOL(left >= right)),
                            LESS => Ok(TokenLiteral::LOX_BOOL(left < right)),
                            LESS_EQUAL => Ok(TokenLiteral::LOX_BOOL(left <= right)),
                            _ => {
                                let msg = String::from("Unrecognized operator passed between two numbers");
                                Err(InterpreterError::OperatorError { operator, msg })
                            },
                        }
                    },
                    // Two strings
                    (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => {
                        match operator.token_type {
                            PLUS => Ok(TokenLiteral::LOX_STRING(left + &right)),
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            },
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            },
                            _ => {
                                let msg = String::from("Non-concatenating operator passed between two strings");
                                Err(InterpreterError::OperatorError { operator, msg })
                            },
                        }
                    },
                    // Two bools
                    (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => {
                        match operator.token_type {
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            },
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            },
                            _ => {
                                let msg = String::from("Non-equality operators passed between two bools");
                                Err(InterpreterError::OperatorError { operator, msg })
                            },
                        }
                    },
                    // Two nils
                    (TokenLiteral::NULL, TokenLiteral::NULL) => {
                        match operator.token_type {
                            EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(TokenLiteral::NULL, TokenLiteral::NULL))),
                            BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(TokenLiteral::NULL, TokenLiteral::NULL))),
                            _ => {
                                let msg = String::from("Non-equality operators passed between two nils");
                                Err(InterpreterError::OperatorError { operator, msg })
                            },
                        }
                    },
                    // Operands of arbitrary, non-equal types
                    (_, _) => {
                        match operator.token_type {
                            EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(false)),
                            BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(true)),
                            _ => {
                                let msg = String::from("Mismatched types operated on");
                                Err(InterpreterError::OperatorError { operator, msg })
                            },
                        }
                    },
                }
            }
            _ => Err(InterpreterError::LiteralError(String::from("Non-binary expression passed to binary visitor")))
        }
    }

    fn visit_unary_expr(expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Unary { operator, right } => {
                let right = Interpreter::accept_expr(right)?;
                match operator.token_type {
                    MINUS => {
                        match right {
                            TokenLiteral::LOX_NUMBER(num) => Ok(TokenLiteral::LOX_NUMBER(-num)),
                            _ => {
                                let msg = String::from("Minus operator used on non-numerical operand");
                                Err(InterpreterError::OperatorError { operator, msg })
                            }
                        }
                    },
                    BANG => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_truthy(right))),
                    _ => Err(InterpreterError::LiteralError(String::from("Unreachable, only two unary operators exist")))
                }
            }
            _ => Err(InterpreterError::LiteralError(String::from("Non-unary expression passed to unary visitor")))
        }
    }

    fn is_truthy(literal: TokenLiteral) -> bool {
        // false and nil are falsy, and everything else is truthy
        match literal {
            TokenLiteral::LOX_BOOL(bool_value) => bool_value,
            TokenLiteral::NULL => false,
            _ => true
        }
    }

    fn is_equal(left: TokenLiteral, right: TokenLiteral) -> bool {
        match (left, right) {
            (TokenLiteral::LOX_NUMBER(left), TokenLiteral::LOX_NUMBER(right)) => left == right,
            (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => left == right,
            (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => left == right,
            (TokenLiteral::NULL, TokenLiteral::NULL) => true,
            (_, _) => false
        }
    }
}