use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::iter::zip;
use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::environment::Environment;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::statement::{Stmt, FunctionObject};
use crate::token_literal::TokenLiteral;

pub struct LoxFunction {
    declaration: Stmt,
    closure: Rc<RefCell<Environment>>
}

impl LoxFunction {
    pub fn new(declaration: Stmt, closure: Rc<RefCell<Environment>>) -> Self {
        match declaration {
            Stmt::Function { .. } => Self { declaration, closure },
            _ => panic!("Non-function declaration passed to LoxFunction constructor")
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match &self.declaration {
            Stmt::Function { ptr } => {
                let FunctionObject { params, body , .. } = ptr.as_ref();
                let mut environment = Environment::new(Some(Rc::clone(&self.closure)));
                for (param_name, value) in zip(params.iter(), arguments.into_iter()) {
                    environment.define(param_name.lexeme.clone(), value);
                }
                interpreter.execute_block(body, Rc::new(RefCell::new(environment)))
            }
            _ => unreachable!()
        }
    }

    fn arity(&self) -> usize {
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