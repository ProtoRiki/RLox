use std::any::Any;
use std::fmt::Binary;
use crate::token::Token;
use crate::token_literal::TokenLiteral;

trait Expr {
    fn accept(&self, visitor: Box<dyn Visitor<_>>);
}

trait Visitor<T> {
    fn visit_assign_expr(&self, expr: &Assign) -> T;
    fn visit_binary_expr(&self, expr: &BinaryEx) -> T;
    fn visit_call_expr(&self, expr: &Call) -> T;
    fn visit_get_expr(&self, expr: &Get) -> T;
    fn visit_grouping_expr(&self, expr: &Grouping) -> T;
    fn visit_literal_expr(&self, expr: &Literal) -> T;
    // fn visit_logical_expr(&self, expr: &Logical) -> T;
    // fn visit_set_expr(&self, expr: &Set) -> T;
    // fn visit_super_expr(&self, expr: &Super) -> T;
    // fn visit_this_expr(&self, expr: &This) -> T;
    // fn visit_unary_expr(&self, expr: &Unary) -> T;
    // fn visit_variable_expr(&self, expr: &Variable) -> T;
}

struct Assign {
    name: Token,
    value: Box<dyn Expr>,
}

impl Assign {
    fn new(name: Token, value: Box<dyn Expr>) -> Self {
        Self { name, value }
    }
}

impl Expr for Assign {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
        visitor.visit_assign_expr(&self);
    }
}

struct BinaryEx {
    left: Box<dyn Expr>,
    operator: Token,
    right: Box<dyn Expr>
}

impl BinaryEx {
    fn new(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>) -> Self {
        Self { left, operator, right }
    }
}

impl Expr for BinaryEx {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
        visitor.visit_binary_expr(&self);
    }
}

struct Call {
    callee: Box<dyn Expr>,
    paren: Token,
    arguments: Vec<Box<dyn Expr>>
}

impl Call {
    fn new(callee: Box<dyn Expr>, paren: Token, arguments: Vec<Box<dyn Expr>>) -> Self {
        Self { callee, paren, arguments }
    }
}

impl Expr for Call {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
        visitor.visit_call_expr(&self);
    }
}

struct Get {
    object: Box<dyn Expr>,
    name: Token
}

impl Get {
    fn new(object: Box<dyn Expr>, name: Token) -> Self {
        Self { object, name }
    }
}

impl Expr for Get {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
        visitor.visit_get_expr(&self);
    }
}

struct Grouping {
    expression: Box<dyn Expr>
}

impl Grouping {
    fn new(expression: Box<dyn Expr>) -> Self {
        Self { expression }
    }
}

impl Expr for Grouping {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
       visitor.visit_grouping_expr(&self);
    }
}

struct Literal {
    value: TokenLiteral
}

impl Literal {
    fn new(value: TokenLiteral) -> Self {
        Self { value }
    }
}

impl Expr for Literal {
    fn accept(&self, visitor: Box<dyn Visitor<_>>) {
       visitor.visit_literal_expr(&self) ;
    }
}