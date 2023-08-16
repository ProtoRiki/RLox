use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use crate::expression::Expr;
use crate::function_object::FunctionObject;
use crate::interpreter::Interpreter;
use crate::lox;
use crate::statement::Stmt;
use crate::token::Token;
use crate::token_literal::TokenLiteral;

// Resolver traverses all AST nodes in a single pass
pub struct Resolver <'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Copy, Clone)]
enum FunctionType {
    NO_FUNCTION,
    INITIALIZER,
    FUNCTION,
    METHOD,
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Copy, Clone)]
enum ClassType {
    NO_CLASS,
    CLASS
}

impl <'a> Resolver <'a> {
    pub fn new (interpreter: &'a mut Interpreter) -> Self {
        Self { interpreter, scopes: Vec::new(), current_function: FunctionType::NO_FUNCTION, current_class: ClassType::NO_CLASS }
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { .. } => self.resolve_block_stmt(stmt),
            Stmt::Class { .. } => self.resolve_class_stmt(stmt),
            Stmt::Expression { .. } => self.resolve_expression_stmt(stmt),
            Stmt::Function { .. } => self.resolve_function_stmt(stmt, FunctionType::FUNCTION),
            Stmt::If { .. } => self.resolve_if_stmt(stmt),
            Stmt::Print { .. } => self.resolve_print_stmt(stmt),
            Stmt::Return { .. } => self.resolve_return_stmt(stmt),
            Stmt::Var { .. } => self.resolve_var_stmt(stmt),
            Stmt::While { .. } => self.resolve_while_stmt(stmt),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { .. } => self.resolve_assign_expr(expr),
            Expr::Binary { .. } => self.resolve_binary_expr(expr),
            Expr::Call { .. } => self.resolve_call_expr(expr),
            Expr::Get { .. } => self.resolve_get_expr(expr),
            Expr::Grouping { .. } => self.resolve_grouping_expr(expr),
            Expr::Literal { .. } => self.resolve_literal_expr(expr),
            Expr::Logical { .. } => self.resolve_logical_expr(expr),
            Expr::Set { .. } => self.resolve_set_expr(expr),
            Expr::This { .. } => self.resolve_this_expr(expr),
            Expr::Unary { .. } => self.resolve_unary_expr(expr),
            Expr::Variable { .. } => self.resolve_var_expr(expr)
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_var(&mut self, name: &Token) {
        if !self.scopes.is_empty() {

            let scope = self.scopes.last_mut().unwrap();

            if scope.contains_key(&name.lexeme) {
                lox::token_error(name, "Already a variable with this name in this scope.")
            }

            // Add to innermost scope to shadow any outer ones
            // Mark "not finished resolving the variable's initializer" with `false`
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define_var(&mut self, name: &Token) {
        if !self.scopes.is_empty() {
            // Should not fail if define is always called after declare
            *self.scopes.last_mut().unwrap().get_mut(&name.lexeme).unwrap() = true;
        }
    }

    pub fn resolve_statements(&mut self, statements: &[Stmt]) {
        for stmt in statements.iter() {
            self.resolve_stmt(stmt)
        }
    }

    fn resolve_block_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_statements(statements);
                self.end_scope();
            }
            _ => unreachable!("Non-block statement passed to block resolver visitor")
        }
    }

    fn resolve_class_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Class { name, methods } => {
                let enclosing_class = self.current_class;

                self.current_class = ClassType::CLASS;
                self.declare_var(name);
                self.define_var(name);

                self.begin_scope();
                // Resolve a 'this' to the local variable in the current method scope
                self.scopes.last_mut().unwrap().insert(String::from("this"), true);

                for method in methods.iter() {
                    let declaration = match method {
                        Stmt::Function { ptr } => {
                            if ptr.name.lexeme == "init" { FunctionType::INITIALIZER } else { FunctionType::METHOD }
                        }
                        _ => unreachable!()
                    };
                    self.resolve_function_stmt(method, declaration);
                }

                self.end_scope();

                self.current_class = enclosing_class;
            }
            _ => unreachable!("Non-class statement passed to class resolver visitor")
        }
    }

    fn resolve_var_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var { name, initializer } => {
                self.declare_var(name);
                self.resolve_expr(initializer);
                self.define_var(name);
            }
            _ => unreachable!("Non-variable statement passed to variable resolver visitor")
        }
    }

    fn resolve_function_stmt(&mut self, stmt: &Stmt, function_type: FunctionType) {
        match stmt {
            Stmt::Function { ptr } => {
                let name = &ptr.as_ref().name;
                self.declare_var(name);
                self.define_var(name);
                self.resolve_function(ptr, function_type)
            }
            _ => unreachable!("Non-function statement passed to function resolver visitor")
        }
    }

    fn resolve_function(&mut self, function: &Rc<FunctionObject>, function_type: FunctionType) {
        let enclosing_function_type = self.current_function;
        self.current_function = function_type;

        self.begin_scope();
        for param in function.params.iter() {
            self.declare_var(param);
            self.define_var(param);
        }
        self.resolve_statements(&function.body);
        self.end_scope();

        self.current_function = enclosing_function_type;
    }

    fn resolve_expression_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression { expression } => self.resolve_expr(expression),
            _ => unreachable!("Non-expression statement passed to expression resolver visitor")
        }
    }

    fn resolve_if_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::If { expression, then_branch, else_branch } => {
                self.resolve_expr(expression);
                self.resolve_stmt(then_branch);
                self.resolve_stmt(else_branch);
            }
            _ => unreachable!("Non-if statement passed to if resolver visitor")
        }
    }

    fn resolve_print_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Print { expression} => self.resolve_expr(expression),
            _ => unreachable!("Non-print statement passed to print resolver visitor")
        }
    }

    fn resolve_return_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::NO_FUNCTION {
                    lox::token_error(keyword, "Can't return from top-level code.");
                }

                match value.deref() {
                    Expr::Literal { value: TokenLiteral::LOX_NULL } => (),
                    _ => {
                        if self.current_function == FunctionType::INITIALIZER {
                            lox::token_error(keyword, "Can't return a value from an initializer");
                        }
                    }
                };

                self.resolve_expr(value)
            },
            _ => unreachable!("Non-return statement passed to return resolver visitor")
        }
    }

    fn resolve_while_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::While { expression, body } => {
                self.resolve_expr(expression);
                self.resolve_stmt(body);
            }
            _ => unreachable!("Non-while statement passed to while resolver visitor")
        }
    }


    fn resolve_local_var(&mut self, expr: &Expr, variable: &Token) {
        // Search from innermost scope outwards to determine the number of scopes
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&variable.lexeme) {
                self.interpreter.resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_var_expr(&mut self, expr: &Expr) {
        let variable = match expr {
            Expr::Variable { name, .. } => { name },
            _ => unreachable!("Non-variable expression passed to variable resolver visitor")
        };

        // Values in scopes map indicate whether a variable has been defined
        if !self.scopes.is_empty() {
            let last_scope = self.scopes.last().unwrap();
            if last_scope.contains_key(&variable.lexeme) && !*last_scope.get(&variable.lexeme).unwrap() {
                // Variable exists in current scope but is undefined (set to `false`)
                lox::token_error(variable, "Can't read local variable in its own initializer.")
            }
        }

        self.resolve_local_var(expr, variable);
    }

    fn resolve_assign_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign { name, value , .. } => {
                self.resolve_expr(value);
                self.resolve_local_var(expr, name);
            }
            _ => unreachable!("Non-assign expression passed to assign resolver visitor")
        }
    }

    fn resolve_binary_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            _ => unreachable!("Non-binary expression passed to binary resolver visitor")
        }
    }

    fn resolve_call_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call { callee, arguments, .. } => {
                self.resolve_expr(callee);
                for arg in arguments.iter() {
                    self.resolve_expr(arg);
                }
            }
            _ => unreachable!("Non-call expression passed to call resolver visitor")
        }
    }

    fn resolve_get_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Get { object, .. } => self.resolve_expr(object),
            _ => unreachable!("Non-get expression passed to get resolver visitor")
        }
    }

    fn resolve_grouping_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Grouping { expression } => self.resolve_expr(expression),
            _ => unreachable!("Non-grouping expression passed to grouping resolver visitor")
        }
    }

    fn resolve_literal_expr(&mut self, expr: &Expr) {
        match expr {
            // Literals contain no variables or sub-expressions to resolve
            Expr::Literal { .. } => (),
            _ => unreachable!("Non-literal expression passed to literal resolver visitor")
        }
    }

    fn resolve_logical_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Logical { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            _ => unreachable!("Non-logical expression passed to logical resolver visitor")
        }
    }

    fn resolve_set_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Set { object, value, .. } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            }
            _ => unreachable!("Non-set expression passed to set resolver visitor")
        }
    }

    fn resolve_this_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::This { name: keyword, .. } => {
                if self.current_class == ClassType::NO_CLASS {
                    lox::token_error(keyword, "Can't use 'this' outside of a class.");
                    return;
                }
                self.resolve_local_var(expr, keyword)
            },
            _ => unreachable!("Non-this expression passed to this-keyword resolver visitor")
        }
    }

    fn resolve_unary_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Unary { right, .. } => self.resolve_expr(right),
            _ => unreachable!("Non-unary expression passed to unary resolver visitor")
        }
    }

}