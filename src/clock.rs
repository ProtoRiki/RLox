use std::time::{SystemTime, UNIX_EPOCH};

use crate::interpreter::InterpreterError;
use crate::token_literal::TokenLiteral::{self, LOX_NUMBER};

pub struct Clock;

impl Clock {
    pub fn time_since_epoch_as_secs() -> Result<TokenLiteral, InterpreterError> {
        Ok(LOX_NUMBER(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64()))
    }

    pub fn arity() -> usize {
        0
    }
}