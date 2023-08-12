use std::fmt::{Display, Formatter};
use crate::function::LoxFunction;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;
use crate::native::NativeFunction;

pub enum LoxCallable {
    Native(NativeFunction),
    UserFunction(LoxFunction),
    // UserMethod(LoxMethod)
}

impl LoxCallable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match self {
            LoxCallable::Native(native) => native.call(interpreter, arguments),
            LoxCallable::UserFunction(function) => function.call(interpreter, arguments)
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Native(native) => native.arity(),
            LoxCallable::UserFunction(function) => function.arity(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Native(_) => write!(f, "<native fn>"),
            LoxCallable::UserFunction(function) => write!(f, "{function}")
        }
    }
}
