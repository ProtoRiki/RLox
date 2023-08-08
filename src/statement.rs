use crate::expression::Expr;
use crate::token::Token;

pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Box<Expr>,
    },

    Print {
        expression: Box<Expr>,
    },

    Var {
        name: Token,
        initializer: Box<Expr>,
    },
}