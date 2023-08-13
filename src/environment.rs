use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::clock::Clock;
use crate::interpreter::InterpreterError;
use crate::native::NativeFunction;
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
                        let err_msg = format!("Undefined variable '{}'", &name.lexeme);
                        Err(InterpreterError::OperatorError { line: name.line, err_msg})
                    }
                }
            }
        }
    }

    pub fn get_at(&mut self, distance: usize, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        if distance == 0 {
            return self.get(name)
        }
        self.ancestor(distance).deref().borrow_mut().get(name)
    }

    fn ancestor(&mut self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut env = self.enclosing.clone().unwrap();
        for _ in 1..distance {
            env = env.deref().take().enclosing.unwrap().clone();
        }
        env
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
                        let err_msg = format!("Undefined variable '{}'.", &name.lexeme);
                        Err(InterpreterError::OperatorError{line: name.line, err_msg})
                    }
                }
            }
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: TokenLiteral) -> Result<(), InterpreterError> {
        if distance == 0 {
            return self.assign(name, value);
        }
        self.ancestor(distance).deref().borrow_mut().assign(name, value)
    }

    pub fn init_native_funcs(&mut self) {
        // Native functions are extensible via implementing the LoxCallable trait object on them
        // Clock
        self.define(String::from("clock"),TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::Native(NativeFunction::NativeClock(Clock)))));
    }
}