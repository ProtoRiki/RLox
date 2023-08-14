use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::clock::Clock;
use crate::interpreter::InterpreterError;
use crate::native::NativeFunction;
use crate::token::Token;
use crate::token_literal::TokenLiteral;

#[derive(Default)]
pub struct Environment {
    pub values: HashMap<String, TokenLiteral>,
}

impl Environment {
    pub fn new () -> Self {
        Self { values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: TokenLiteral) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                let err_msg = format!("Undefined variable '{}'", &name.lexeme);
                Err(InterpreterError::OperatorError { line: name.line, err_msg})
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: TokenLiteral) -> Result<(), InterpreterError> {
        match self.values.get_mut(&name.lexeme) {
            Some(val) => {
                *val = value;
                Ok(())
            }
            None => {
                let err_msg = format!("Undefined variable '{}'.", &name.lexeme);
                Err(InterpreterError::OperatorError{line: name.line, err_msg})
            }
        }
    }

    pub fn init_native_funcs(&mut self) {
        // Native functions are extensible via implementing the LoxCallable trait object on them
        // Clock
        self.define(String::from("clock"),TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::Native(NativeFunction::NativeClock(Clock)))));
    }
}