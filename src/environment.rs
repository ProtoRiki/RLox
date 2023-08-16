use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::clock::Clock;
use crate::interpreter::InterpreterError;
use crate::native_function::NativeFunction;
use crate::token_literal::TokenLiteral;
use crate::token::Token;

#[derive(Default)]
pub struct Environment {
    values: RefCell<HashMap<String, TokenLiteral>>,
    pub enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new (enclosing: Option<Rc<Environment>>) -> Self {
        Self { values: RefCell::new(HashMap::new()), enclosing }
    }

    pub fn define(&self, name: String, value: TokenLiteral) {
        self.values.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        match self.values.borrow().get(&name.lexeme) {
            Some(val) => Ok(val.clone()),
            None => {
                match &self.enclosing {
                    Some(enclosing) => enclosing.get(name),
                    None => {
                        let err_msg = format!("Undefined variable '{}'", &name.lexeme);
                        Err(InterpreterError::OperatorError { line: name.line, err_msg})
                    }
                }
            }
        }
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        if distance == 0 {
            return self.get(name)
        }
        self.ancestor(distance).deref().get(name)
    }

    fn ancestor(&self, distance: usize) -> Rc<Environment> {
        let mut env = self.enclosing.clone().unwrap();
        for _ in 1..distance {
            env = env.enclosing.clone().unwrap()
        }
        env
    }

    pub fn assign(&self, name: &Token, value: TokenLiteral) -> Result<(), InterpreterError> {
        match self.values.borrow_mut().get_mut(&name.lexeme) {
            Some(val) => {
                *val = value;
                Ok(())
            }
            None => {
                match &self.enclosing {
                    Some(enclosing) => enclosing.assign(name, value),
                    None => {
                        let err_msg = format!("Undefined variable '{}'.", &name.lexeme);
                        Err(InterpreterError::OperatorError{line: name.line, err_msg})
                    }
                }
            }
        }
    }

    pub fn assign_at(&self, distance: usize, name: &Token, value: TokenLiteral) -> Result<(), InterpreterError> {
        if distance == 0 {
            return self.assign(name, value);
        }
        self.ancestor(distance).deref().assign(name, value)
    }

    pub fn init_native_funcs(&self) {
        // Native functions are extensible via implementing the LoxCallable trait object on them
        // Clock
        self.define(String::from("clock"),TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::Native(NativeFunction::NativeClock(Clock)))));
    }
}