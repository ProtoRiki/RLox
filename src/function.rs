use std::fmt::{Display, Formatter};
use std::iter::zip;

use crate::environment::Environment;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::statement::Stmt;
use crate::function_object::FunctionObject;
use crate::token_literal::TokenLiteral;

pub struct LoxFunction {
    declaration: Stmt,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Self {
        match declaration {
            Stmt::Function { .. } => Self { declaration },
            _ => panic!("Non-function declaration passed to LoxFunction constructor")
        }
    }
}

impl LoxFunction {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match &self.declaration {
            Stmt::Function { ptr } => {
                let FunctionObject { params, body , .. } = ptr.as_ref();
                let mut environment = Environment::new();
                for (param_name, value) in zip(params.iter(), arguments.into_iter()) {
                    environment.define(param_name.lexeme.clone(), value);
                }

                // Update internal call stack
                interpreter.environments.push(Box::new(environment));
                let res = interpreter.execute_block(body);
                interpreter.environments.pop();
                res
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
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.declaration {
            Stmt::Function { ptr } => write!(f, "<fn {}>", ptr.as_ref().name.lexeme),
            _ => unreachable!()
        }
    }
}