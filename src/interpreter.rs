use std::mem;
use std::rc::Rc;
use std::cell::RefCell;

use crate::expression::Expr::{self, *};
use crate::lox;
use crate::statement::Stmt::{self, *};
use crate::token_literal::TokenLiteral;
use crate::token_type::TokenType::*;
use crate::environment::Environment;
use crate::function::LoxFunction;

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
    pub global: Rc<RefCell<Environment>>
}

pub enum InterpreterError {
    LiteralError(String),
    OperatorError { line: i32, msg: String },
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::new(None)));
        (*global).borrow_mut().init_native_funcs();
        Self { env: Rc::clone(&global), global}
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements.iter() {
            if let Err(error) = self.accept_statement(statement) {
                lox::runtime_error(&error);
                return;
            }
        }
    }

    fn accept_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Assign { .. } => self.visit_assign_expr(expr),
            Binary { .. } => self.visit_binary_expr(expr),
            Call { .. } => self.visit_call_expr(expr),
            Grouping { .. } => self.visit_grouping_expr(expr),
            Literal { .. } => self.visit_literal_expr(expr),
            Logical { .. } => self.visit_logical_expr(expr),
            Unary { .. } => self.visit_unary_expr(expr),
            Variable { .. } => self.visit_variable_expr(expr),
        }
    }

    fn accept_statement(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Block { .. } => self.visit_block_stmt(stmt),
            Expression { .. } => self.visit_expression_stmt(stmt),
            Function { .. } => self.visit_function_stmt(stmt),
            Print { .. } => self.visit_print_stmt(stmt),
            Return { .. } => self.visit_return_stmt(stmt),
            Var { .. } => self.visit_var_stmt(stmt),
            If { .. } => self.visit_if_stmt(stmt),
            While { .. } => self.visit_while_stmt(stmt),
        }
    }


    fn visit_block_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Block { statements } => {
                let env = Rc::new(RefCell::new(Environment::new(Some(Rc::clone(&self.env)))));
                self.execute_block(statements, env)?;
            },
            _ => {
                let msg = String::from("Non-block statement passed to block visitor");
                return Err(InterpreterError::LiteralError(msg));
            }
        }
        Ok(TokenLiteral::NULL)
    }
    pub fn execute_block(&mut self, statements: &[Stmt], environment: Rc<RefCell<Environment>>) -> Result<TokenLiteral, InterpreterError> {
        let previous = mem::replace(&mut self.env, environment);
        for statement in statements.iter() {
            match self.accept_statement(statement) {
                Ok(literal) => match literal {
                    // Exit block early on reaching return
                    TokenLiteral::NULL => (),
                    _ => {
                        self.env = previous;
                        return Ok(literal);
                    }
                }
                Err(error) => {
                    self.env = previous;
                    return Err(error);
                }
            }
        }
        self.env = previous;
        Ok(TokenLiteral::NULL)
    }

    fn visit_expression_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Expression { expression } => {
                self.accept_expr(expression)?;
                Ok(TokenLiteral::NULL)
            },
            _ => {
                let msg = String::from("Non-expression statement passed to expr visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Print { expression } => {
                let value = self.accept_expr(expression)?;
                println!("{}", value);
                Ok(TokenLiteral::NULL)
            }
            _ => {
                let msg = String::from("Non-print statement passed to print visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_var_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Var { name, initializer } => {
                let value = self.accept_expr(initializer)?;
                self.env.borrow_mut().define(name.lexeme.clone(), value);
                Ok(TokenLiteral::NULL)
            }
            _ => {
                let msg = String::from("Non-var statement passed to var visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_if_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            If { expression, then_branch, else_branch} => {
                match Interpreter::is_truthy(&self.accept_expr(expression)?) {
                    true => self.accept_statement(then_branch),
                    false => self.accept_statement(else_branch),
                }
            }
            _ => {
                let msg = String::from("Non-if statement passed to if visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_while_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            While { expression, body } => {
                while Interpreter::is_truthy(&self.accept_expr(expression)?) {
                    self.accept_statement(body)?;
                }
                Ok(TokenLiteral::NULL)
            }
            _ => {
                let msg = String::from("Non-while statement passed to while visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_function_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Function { ptr } => {
                let function_obj = Function { ptr: Rc::clone(ptr) };
                let curr_env = self.env.clone();
                let function = Rc::new(LoxFunction::new(function_obj, curr_env));
                self.env.borrow_mut().define(ptr.as_ref().name.lexeme.clone(), TokenLiteral::LOX_CALLABLE(function));
                Ok(TokenLiteral::NULL)
            }
            _ => {
                let msg= String::from("Non-function statement passed to function visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_return_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Return { keyword: _keyword, value } => {
                let value = self.accept_expr(value)?;
                Ok(value)
            }
            _ => {
                let msg = String::from("Non-return statement passed to return visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_literal_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Literal { value } => Ok(value.clone()),
            _ => {
                let msg = String::from("Non-literal expression passed to literal visitor");
                Err(InterpreterError::LiteralError(msg))
            },
        }
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Logical { left, operator, right } => {
                let left = self.accept_expr(left)?;
                match (Interpreter::is_truthy(&left), operator.token_type) {
                    // Short-circuit
                    (true, OR) | (false, AND) => Ok(left),
                    (_, _) => self.accept_expr(right)
                }
            },
            _ => {
                let msg = String::from("Non-logical expression passed to logical visitor");
                Err(InterpreterError::LiteralError(msg))
            },
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Grouping { expression } => self.accept_expr(expression),
            _ => {
                let msg = String::from("Non-group expression passed to group visitor");
                Err(InterpreterError::LiteralError(msg))
            } 
        }
    }
    fn visit_binary_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
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
                                Err(InterpreterError::OperatorError { line: operator.line, msg })
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
                                Err(InterpreterError::OperatorError { line: operator.line, msg })
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
                                Err(InterpreterError::OperatorError { line: operator.line, msg })
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
                            Err(InterpreterError::OperatorError { line: operator.line, msg })
                        }
                    },
                    // Operands of arbitrary, non-equal types
                    (_, _) => match operator.token_type {
                        EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(false)),
                        BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(true)),
                        _ => {
                            let msg = String::from("Mismatched types operated on");
                            Err(InterpreterError::OperatorError { line: operator.line, msg })
                        }
                    },
                }
            }
            _ => {
                let msg = String::from("Non-binary expression passed to binary visitor");
                Err(InterpreterError::LiteralError(msg))
            },
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Call { callee, paren, arguments } => {

                let callee = self.accept_expr(callee)?;
                let mut parameters = Vec::with_capacity(arguments.len());
                for arg in arguments.iter() {
                    parameters.push(self.accept_expr(arg)?)
                }

                match callee {
                    TokenLiteral::LOX_CALLABLE(callable) => {
                        match callable.arity() == parameters.len() {
                            true => callable.call(self, parameters),
                            false => {
                                let msg = format!("Expected {} arguments but got {}.", callable.arity(), parameters.len());
                                Err(InterpreterError::OperatorError { line: paren.line, msg})
                            }
                        }
                    }
                    _ => {
                        let msg = String::from("Can only call functions and classes.");
                        Err(InterpreterError::OperatorError { line: paren.line, msg})
                    }
                }
            }
            _ => {
                let msg = String::from("Non-call expression passed to call visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Unary { operator, right } => {
                let right = self.accept_expr(right)?;
                match operator.token_type {
                    MINUS => match right {
                        TokenLiteral::LOX_NUMBER(num) => Ok(TokenLiteral::LOX_NUMBER(-num)),
                        _ => {
                            let msg = String::from("Minus operator used on non-numerical operand");
                            Err(InterpreterError::OperatorError { line: operator.line, msg })
                        }
                    },
                    BANG => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_truthy(&right))),
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

    fn visit_variable_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Variable { name } => self.env.borrow_mut().get(name),
            _ => {
                let msg = String::from("Non-variable expression passed to variable visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn visit_assign_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Assign { name, value } => {
                let value = self.accept_expr(value)?;
                self.env.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            _ => {
                let msg = String::from("Non-assignment expression passed to assignment visitor");
                Err(InterpreterError::LiteralError(msg))
            }
        }
    }

    fn is_truthy(literal: &TokenLiteral) -> bool {
        // false and nil are falsy, and everything else is truthy
        match literal {
            TokenLiteral::LOX_BOOL(bool_value) => *bool_value,
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
