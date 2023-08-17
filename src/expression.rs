use crate::token::Token;
use crate::token_literal::TokenLiteral;

pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
        id: usize
    },

    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    Get {
        object: Box<Expr>,
        name: Token,
        id: usize,
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: TokenLiteral,
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
        id: usize,
    },

    Super {
        keyword: Token,
        method: Token,
        id: usize,
    },

    This {
        name: Token,
        id: usize,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },

    Variable {
        name: Token,
        id: usize,
    },
}