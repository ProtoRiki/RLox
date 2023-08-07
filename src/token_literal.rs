#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum TokenLiteral {
    LOX_STRING(String),
    LOX_NUMBER(f64),
    LOX_BOOL(bool),
    NULL
}

impl TokenLiteral {
    pub fn to_string(&self) -> String {
        match self {
            TokenLiteral::LOX_STRING(value) => value.clone(),
            TokenLiteral::LOX_NUMBER(number) => format!("{}", number),
            TokenLiteral::LOX_BOOL(boolean) => format!("{}", boolean),
            TokenLiteral::NULL => String::from("nil"),
        }
    }
}