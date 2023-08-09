use crate::expression::Expr;
use crate::token::Token;

pub enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },

    Expression {
        expression: Box<Expr>,
    },

    If {
        expression: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Box<Stmt>,
    },

    Print {
        expression: Box<Expr>,
    },

    Var {
        name: Token,
        initializer: Box<Expr>,
    },
}