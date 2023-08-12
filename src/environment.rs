use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::clock::Clock;
use crate::interpreter::InterpreterError;
use crate::token_literal::TokenLiteral;
use crate::token::Token;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, TokenLiteral>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new (enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self { values: HashMap::new(), enclosing }
    }

    pub fn define(&mut self, name: String, value: TokenLiteral) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                match self.enclosing.as_mut() {
                    Some(enclosing) => enclosing.borrow_mut().get(name),
                    None => {
                        let msg = format!("Undefined variable '{}'", &name.lexeme);
                        Err(InterpreterError::OperatorError { line: name.line, msg})
                    }
                }
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
                match self.enclosing.as_mut() {
                    Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                    None => {
                        let msg = format!("Undefined variable '{}'.", &name.lexeme);
                        Err(InterpreterError::OperatorError{line: name.line, msg})
                    }
                }
            }
        }
    }

    pub fn init_native_funcs(&mut self) {
        // Native functions are extensible via implementing the LoxCallable trait object on them
        // Clock
        self.define(String::from("clock"), TokenLiteral::LOX_CALLABLE(Rc::new(Clock)));
    }
}