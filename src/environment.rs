use std::collections::HashMap;
use crate::interpreter::InterpreterError;
use crate::token_literal::TokenLiteral;
use crate::token::Token;

pub struct Environment {
    values: HashMap<String, TokenLiteral>
}

impl Environment {
    pub fn new () -> Self {
        Self { values: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: TokenLiteral) {
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: Token) -> Result<TokenLiteral, InterpreterError>{
        // let msg = format!("Undefined variable '{}'", &name.lexeme);
        // let error = InterpreterError::OperatorError { operator: name, msg};
        self.values.get(&name.lexeme).ok_or_else(|| {
            let msg = format!("Undefined variable '{}'", &name.lexeme);
            InterpreterError::OperatorError { operator: name, msg} }
        ).cloned()
    }
}