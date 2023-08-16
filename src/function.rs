use std::fmt::{Display, Formatter};
use std::iter::zip;
use std::rc::Rc;
use crate::class_instance::LoxInstance;

use crate::environment::Environment;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::statement::Stmt;
use crate::function_object::FunctionObject;
use crate::token::Token;
use crate::token_literal::TokenLiteral;
use crate::token_type::TokenType;

pub struct LoxFunction {
    declaration: Stmt,
    closure: Rc<Environment>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: Rc<Environment>, is_initializer: bool) -> Self {
        match declaration {
            Stmt::Function { .. } => Self { declaration, closure, is_initializer },
            _ => unreachable!("Non-function declaration passed to LoxFunction constructor")
        }
    }
}

impl LoxFunction {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match &self.declaration {
            Stmt::Function { ptr } => {
                let FunctionObject { params, body , .. } = ptr.as_ref();
                let environment = Environment::new(Some(Rc::clone(&self.closure)));
                for (param_name, value) in zip(params.iter(), arguments.into_iter()) {
                    environment.define(param_name.lexeme.clone(), value);
                }
                let block_return_val = interpreter.execute_block(body, Rc::new(environment));

                // Force-return `this` if calling constructor
                if self.is_initializer {
                    // get_at takes a &Token, but we only care that its lexeme is 'this'
                    let dummy_token = Token { token_type: TokenType::EOF, lexeme: String::from("this"), line: 0, literal: TokenLiteral::LOX_NULL };
                    return self.closure.get_at(0, &dummy_token);
                }

                // Stop propagation of a return value
                if let Err(InterpreterError::Return(literal)) = block_return_val {
                    return Ok(literal)
                }


                // Propagate interpreter errors only from here on
                Ok(block_return_val?)
            }
            _ => unreachable!()
        }
    }

    pub fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function { ptr } => ptr.as_ref().params.len(),
            _ => unreachable!()
        }
    }

    pub fn bind(&self, instance: Rc<LoxInstance>) -> LoxFunction {
        let environment = Environment::new(Some(Rc::clone(&self.closure)));
        environment.define(String::from("this"), TokenLiteral::LOX_INSTANCE(instance));
        match &self.declaration {
            Stmt::Function { ptr } => {
                let declaration = Stmt::Function { ptr: Rc::clone(ptr) };
                LoxFunction { closure: Rc::new(environment), declaration, is_initializer: self.is_initializer }
            }
            _ => unreachable!()
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.declaration {
            Stmt::Function { ptr } => write!(f, "<fn {}>", ptr.as_ref().name.lexeme),
            _ => unreachable!()
        }
    }
}