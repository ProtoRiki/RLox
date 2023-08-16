use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::callable::LoxCallable;

use crate::class::LoxClass;
use crate::interpreter::InterpreterError;
use crate::token::Token;
use crate::token_literal::TokenLiteral;

pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: RefCell<HashMap<String, TokenLiteral>>,
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self { class, fields: RefCell::new(HashMap::new()) }
    }

    pub fn get(&self, self_rc: Rc<Self>, name: &Token) -> Result<TokenLiteral, InterpreterError> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            return Ok(self.fields.borrow().get(&name.lexeme).unwrap().clone());
        }

        if let Some(method) = self.class.find_method(&name.lexeme) {
            let function = method.bind(self_rc);
            let function = TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::UserFunction(Rc::new(function))));
            return Ok(function);
        }

        let err_msg = format!("Undefined property '{}'", name.lexeme);
        Err(InterpreterError::OperatorError {err_msg, line: name.line})
    }

    pub fn set(&self, name: &Token, value: TokenLiteral) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}