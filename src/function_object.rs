use crate::statement::Stmt;
use crate::token::Token;

pub struct FunctionObject {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>
}