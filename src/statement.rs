use crate::expression::Expr;

pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
}