use crate::expression::Expr::{self, *};
use crate::lox;
use crate::statement::Stmt::{self, *};
use crate::token::Token;
use crate::token_literal::TokenLiteral;
use crate::token_type::TokenType::*;
use crate::environment::Environment;

pub struct Interpreter {
    env: Environment
}

pub enum InterpreterError {
    LiteralError(String),
    OperatorError { operator: Token, msg: String },
}

impl Interpreter {
    pub fn new() -> Self {
        Self { env: Environment::new() }
    }
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements.into_iter() {
            if let Err(error) = self.accept_statement(statement) {
                lox::runtime_error(&error);
                return;
            }
        }
    }

    fn accept_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Binary {
                left,
                operator,
                right,
            } => self.visit_binary_expr(Box::from(Binary {
                left,
                operator,
                right,
            })),

            Grouping { expression } => {
                self.visit_grouping_expr(Box::from(Grouping { expression }))
            },

            Literal { value } => self.visit_literal_expr(Box::from(Literal { value })),

            Unary { operator, right } => {
                self.visit_unary_expr(Box::from(Unary { operator, right }))
            },

            Variable { name } => self.visit_variable_expr(Box::from(Variable { name })),
        }
    }

    fn accept_statement(&mut self, stmt: Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Expression { expression } => {
                self.visit_expression_stmt(Expression { expression })
            },
            Print { expression } => self.visit_print_stmt(Print { expression }),
            Var { name, initializer } => self.visit_var_stmt(Var { name, initializer }),
        }
    }

    fn visit_expression_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Expression { expression } => self.accept_expr(expression)?,
            _ => {
                let msg = String::from("Non-expression statement passed to expr visitor");
                return Err(InterpreterError::LiteralError(msg));
            }
        };
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Print { expression } => {
                let value = self.accept_expr(expression)?;
                Ok(println!("{}", value))
            }
            _ => {
                let msg = String::from("Non-print statement passed to print visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_var_stmt(&mut self, stmt: Stmt) -> Result<(), InterpreterError> {
        match stmt {
            Var { name, initializer } => {
                let value = self.accept_expr(initializer)?;
                self.env.define(name.lexeme, value);
                Ok(())
            }
            _ => {
                let msg = String::from("Non-var statement passed to var visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_literal_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Literal { value } => Ok(value),
            _ => Err(InterpreterError::LiteralError(String::from(
                "Non-literal expression passed to literal visitor",
            ))),
        }
    }

    fn visit_grouping_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Grouping { expression } => self.accept_expr(expression),
            _ => Err(InterpreterError::LiteralError(String::from(
                "Non-group expression passed to group visitor",
            ))),
        }
    }
    fn visit_binary_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Binary {
                left,
                operator,
                right,
            } => {
                // Recursively evaluate operands until they are usable literals
                let left = self.accept_expr(left)?;
                let right = self.accept_expr(right)?;
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
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_NUMBER(left);
                                let right = TokenLiteral::LOX_NUMBER(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            GREATER => Ok(TokenLiteral::LOX_BOOL(left > right)),
                            GREATER_EQUAL => Ok(TokenLiteral::LOX_BOOL(left >= right)),
                            LESS => Ok(TokenLiteral::LOX_BOOL(left < right)),
                            LESS_EQUAL => Ok(TokenLiteral::LOX_BOOL(left <= right)),
                            _ => {
                                let msg = String::from(
                                    "Unrecognized operator passed between two numbers",
                                );
                                Err(InterpreterError::OperatorError { operator, msg })
                            }
                        }
                    }
                    // Two strings
                    (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => {
                        match operator.token_type {
                            PLUS => Ok(TokenLiteral::LOX_STRING(left + &right)),
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            _ => {
                                let msg = String::from(
                                    "Non-concatenating operator passed between two strings",
                                );
                                Err(InterpreterError::OperatorError { operator, msg })
                            }
                        }
                    }
                    // Two bools
                    (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => {
                        match operator.token_type {
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            _ => {
                                let msg =
                                    String::from("Non-equality operators passed between two bools");
                                Err(InterpreterError::OperatorError { operator, msg })
                            }
                        }
                    }
                    // Two nils
                    (TokenLiteral::NULL, TokenLiteral::NULL) => match operator.token_type {
                        EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(
                            TokenLiteral::NULL,
                            TokenLiteral::NULL,
                        ))),
                        BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(
                            TokenLiteral::NULL,
                            TokenLiteral::NULL,
                        ))),
                        _ => {
                            let msg =
                                String::from("Non-equality operators passed between two nils");
                            Err(InterpreterError::OperatorError { operator, msg })
                        }
                    },
                    // Operands of arbitrary, non-equal types
                    (_, _) => match operator.token_type {
                        EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(false)),
                        BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(true)),
                        _ => {
                            let msg = String::from("Mismatched types operated on");
                            Err(InterpreterError::OperatorError { operator, msg })
                        }
                    },
                }
            }
            _ => Err(InterpreterError::LiteralError(String::from(
                "Non-binary expression passed to binary visitor",
            ))),
        }
    }

    fn visit_unary_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Unary { operator, right } => {
                let right = self.accept_expr(right)?;
                match operator.token_type {
                    MINUS => match right {
                        TokenLiteral::LOX_NUMBER(num) => Ok(TokenLiteral::LOX_NUMBER(-num)),
                        _ => {
                            let msg = String::from("Minus operator used on non-numerical operand");
                            Err(InterpreterError::OperatorError { operator, msg })
                        }
                    },
                    BANG => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_truthy(right))),
                    _ => {
                        let msg = String::from("Unreachable, only two unary operators exist");
                        Err(InterpreterError::LiteralError(msg))
                    },
                }
            }
            _ => {
                let msg = String::from("Non-unary expression passed to unary visitor");
                Err(InterpreterError::LiteralError(msg))
            },
        }
    }

    fn visit_variable_expr(&mut self, expr: Box<Expr>) -> Result<TokenLiteral, InterpreterError> {
        match *expr {
            Variable { name } => self.env.get(name),
            _ => {
                let msg = String::from("Non-variable expression passed to variable visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn is_truthy(literal: TokenLiteral) -> bool {
        // false and nil are falsy, and everything else is truthy
        match literal {
            TokenLiteral::LOX_BOOL(bool_value) => bool_value,
            TokenLiteral::NULL => false,
            _ => true,
        }
    }

    fn is_equal(left: TokenLiteral, right: TokenLiteral) -> bool {
        match (left, right) {
            (TokenLiteral::LOX_NUMBER(left), TokenLiteral::LOX_NUMBER(right)) => left == right,
            (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => left == right,
            (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => left == right,
            (TokenLiteral::NULL, TokenLiteral::NULL) => true,
            (_, _) => false,
        }
    }
}
