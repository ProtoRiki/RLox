use std::fmt::{Display, Formatter};
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;

pub trait LoxCallable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError>;
    fn arity(&self) -> usize;
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<abstract callable object>")
    }
}
