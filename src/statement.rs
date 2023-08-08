use crate::expression::Expr;
use crate::token::Token;

pub enum Stmt {
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