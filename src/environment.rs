use std::collections::HashMap;
use crate::interpreter::InterpreterError;
use crate::token_literal::TokenLiteral;
use crate::token::Token;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, TokenLiteral>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new (enclosing: Option<Box<Environment>>) -> Self {
        Self { values: HashMap::new(), enclosing }
    }

    pub fn define(&mut self, name: String, value: TokenLiteral) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: Token) -> Result<TokenLiteral, InterpreterError> {
        match self.values.get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                match self.enclosing.as_mut() {
                    Some(enclosing) => enclosing.get(name),
                    None => {
                        let msg = format!("Undefined variable '{}'", &name.lexeme);
                        Err(InterpreterError::OperatorError { operator: name, msg})
                    }
                }
            }
        }
    }

    pub fn assign(&mut self, name: Token, value: TokenLiteral) -> Result<(), InterpreterError> {
        match self.values.get_mut(&name.lexeme) {
            Some(val) => {
                *val = value;
                Ok(())
            }
            None => {
                match self.enclosing.as_mut() {
                    Some(enclosing) => enclosing.assign(name, value),
                    None => {
                        let msg = format!("Undefined variable '{}'.", &name.lexeme);
                        Err(InterpreterError::OperatorError{operator: name, msg})
                    }
                }
            }
        }
    }
}