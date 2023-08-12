use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::callable::LoxCallable;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;
use crate::token_literal::TokenLiteral::LOX_NUMBER;

pub struct Clock;

impl LoxCallable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        Ok(LOX_NUMBER(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}