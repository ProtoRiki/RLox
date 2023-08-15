use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;
use crate::callable::LoxCallable;

use crate::class_instance::LoxInstance;
use crate::function::LoxFunction;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::token_literal::TokenLiteral;

pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        if let Some(TokenLiteral::LOX_CALLABLE(constructor)) = arguments.last() {
            if let LoxCallable::ClassConstructor(class) = constructor.deref() {
                let instance = LoxInstance::new(Rc::clone(class));
                return Ok(TokenLiteral::LOX_INSTANCE(Rc::new(instance)));
            }
        }
        unreachable!("Last argument to class constructor call must be pointer to class object")
    }

    pub fn arity(&self) -> usize {
        0
    }

    pub fn find_method(&self, key: &String) -> Option<TokenLiteral> {
        if self.methods.contains_key(key) {
            let method = self.methods.get(key).unwrap();
            return Some(TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::UserFunction(Rc::clone(method)))));
        }
        None
    }
}

impl Display for LoxClass{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}