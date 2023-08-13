use crate::clock::Clock;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;

pub enum NativeFunction {
    NativeClock(Clock)
}

impl NativeFunction {
    pub fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        match self {
            NativeFunction::NativeClock(_)=> Clock::time_since_epoch_as_secs()
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            NativeFunction::NativeClock(_) => Clock::arity()
        }
    }
}