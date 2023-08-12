use crate::clock::Clock;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::native::NativeFunction::CLOCK;
use crate::token_literal::TokenLiteral;

pub enum NativeFunction {
    CLOCK(Clock)
}

impl NativeFunction {
    pub fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match self {
            CLOCK(_)=> Clock::time_since_epoch_as_secs()
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            CLOCK(_) => Clock::arity()
        }
    }
}