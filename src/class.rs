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
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: String, superclass: Option<Rc<LoxClass>>, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, superclass, methods }
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<TokenLiteral>) -> Result<TokenLiteral, InterpreterError> {
        if let Some(TokenLiteral::LOX_CALLABLE(constructor)) = arguments.last() {
            if let LoxCallable::ClassConstructor(class) = constructor.deref() {
                let instance = LoxInstance::new(Rc::clone(class));
                let instance = Rc::new(instance);

                let initializer = self.find_method(&String::from("init"));
                match initializer {
                    // Immediately bind and invoke the constructor
                    Some(initializer) => initializer.bind(Rc::clone(&instance)).call(interpreter, arguments)?,
                    None => TokenLiteral::LOX_NULL,
                };

                return Ok(TokenLiteral::LOX_INSTANCE(instance));
            }
        }
        unreachable!("Last argument to class constructor call must be pointer to class object")
    }

    pub fn arity(&self) -> usize {
        match self.find_method(&String::from("init")) {
            Some(initializer) => initializer.arity(),
            None => 0
        }
    }

    pub fn find_method(&self, key: &String) -> Option<Rc<LoxFunction>> {
        if self.methods.contains_key(key) {
            let method = self.methods.get(key).unwrap();
            return Some(Rc::clone(method));
        }
        if self.superclass.is_some() {
            return self.superclass.as_ref().unwrap().find_method(key);
        }
        None
    }
}

impl Display for LoxClass{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}