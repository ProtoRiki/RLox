use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::class::LoxClass;
use crate::function::LoxFunction;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;
use crate::native_function::NativeFunction;

pub enum LoxCallable {
    Native(NativeFunction),
    UserFunction(Rc<LoxFunction>),
    ClassConstructor(Rc<LoxClass>),
}

impl LoxCallable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match self {
            LoxCallable::Native(native) => native.call(interpreter, arguments),
            LoxCallable::UserFunction(function) => function.call(interpreter, arguments),
            LoxCallable::ClassConstructor(class) => class.call(interpreter, arguments),
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Native(native) => native.arity(),
            LoxCallable::UserFunction(function) => function.arity(),
            LoxCallable::ClassConstructor(class) => class.arity(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxCallable::Native(_) => write!(f, "<native fn>"),
            LoxCallable::UserFunction(function) => write!(f, "{function}"),
            LoxCallable::ClassConstructor(lox_class) => write!(f, "{lox_class}")
        }
    }
}
