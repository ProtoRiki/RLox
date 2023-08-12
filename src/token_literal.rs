use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::callable::LoxCallable;

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum TokenLiteral {
    LOX_BOOL(bool),
    LOX_CALLABLE(Rc<LoxCallable>),
    LOX_NUMBER(f64),
    LOX_STRING(Rc<String>),
    NULL
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::LOX_STRING(value) => write!(f, "{value}"),
            TokenLiteral::LOX_NUMBER(number) => write!(f, "{}", number),
            TokenLiteral::LOX_BOOL(boolean) => write!(f, "{}", boolean),
            TokenLiteral::NULL => write!(f, "nil"),
            TokenLiteral::LOX_CALLABLE(callable) => write!(f, "{}", callable),
        }
    }
}