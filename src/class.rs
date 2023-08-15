use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;
use crate::callable::LoxCallable;

use crate::class_instance::LoxInstance;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;

#[derive(Clone)]
pub struct LoxClass {
    name: String
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        if let Some(TokenLiteral::LOX_CALLABLE(constructor)) = arguments.last() {
            if let LoxCallable::ClassConstructor(class) = constructor.deref() {
                let instance = LoxInstance::new(Rc::clone(class));
                return Ok(TokenLiteral::LOX_INSTANCE(Rc::new(instance)));
            }
        }
        unreachable!("Last argument to class constructor call must be pointer to class object")
    }

    pub fn arity(&self) -> usize {
        0
    }
}

impl Display for LoxClass{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}