use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum TokenLiteral {
    LOX_STRING(String),
    LOX_NUMBER(f64),
    LOX_BOOL(bool),
    NULL
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::LOX_STRING(value) => write!(f, "{value}"),
            TokenLiteral::LOX_NUMBER(number) => write!(f, "{}", number),
            TokenLiteral::LOX_BOOL(boolean) => write!(f, "{}", boolean),
            TokenLiteral::NULL => write!(f, "nil"),
        }
    }
}