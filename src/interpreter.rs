use std::mem;
use std::rc::Rc;
use std::collections::HashMap;
use std::ops::Deref;

use crate::callable::LoxCallable;
use crate::class::LoxClass;
use crate::class_instance::LoxInstance;
use crate::environment::Environment;
use crate::expression::Expr::{self, *};
use crate::function::LoxFunction;
use crate::lox;
use crate::statement::Stmt::{self, *};
use crate::token::Token;
use crate::token_literal::TokenLiteral;
use crate::token_type::TokenType::*;

pub struct Interpreter {
    pub global_env: Rc<Environment>,
    pub curr_env: Rc<Environment>,
    pub locals: HashMap<usize, usize>
}

pub enum InterpreterError {
    OperatorError { line: i32, err_msg: String },
    Return(TokenLiteral),
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Environment::new(None);
        global.init_native_funcs();
        let global = Rc::new(global);
        Self { curr_env: Rc::clone(&global), global_env: global, locals: HashMap::new() }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements.iter() {
            if let Err(error) = self.accept_statement(statement) {
                lox::runtime_error(&error);
                return;
            }
        }
    }

    fn accept_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Assign { .. } => self.visit_assign_expr(expr),
            Binary { .. } => self.visit_binary_expr(expr),
            Call { .. } => self.visit_call_expr(expr),
            Get { .. } => self.visit_get_expr(expr),
            Grouping { .. } => self.visit_grouping_expr(expr),
            Literal { .. } => self.visit_literal_expr(expr),
            Logical { .. } => self.visit_logical_expr(expr),
            Set { .. } => self.visit_set_expr(expr),
            Super { .. } => self.visit_super_expr(expr),
            This { .. } => self.visit_this_expr(expr),
            Unary { .. } => self.visit_unary_expr(expr),
            Variable { .. } => self.visit_variable_expr(expr),
        }
    }

    fn accept_statement(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Block { .. } => self.visit_block_stmt(stmt),
            Class { .. } => self.visit_class_stmt(stmt),
            Expression { .. } => self.visit_expression_stmt(stmt),
            Function { .. } => self.visit_function_stmt(stmt),
            Print { .. } => self.visit_print_stmt(stmt),
            Return { .. } => self.visit_return_stmt(stmt),
            Var { .. } => self.visit_var_stmt(stmt),
            If { .. } => self.visit_if_stmt(stmt),
            While { .. } => self.visit_while_stmt(stmt),
        }
    }


    fn visit_block_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Block { statements } => {
                let env = Rc::new(Environment::new(Some(Rc::clone(&self.curr_env))));
                self.execute_block(statements, env)?;
            }
            _ => unreachable!("Non-block statement passed to block visitor")
        }
        Ok(TokenLiteral::LOX_NULL)
    }

    pub fn execute_block(&mut self, statements: &[Stmt], environment: Rc<Environment>) -> Result<TokenLiteral, InterpreterError> {
        let previous = mem::replace(&mut self.curr_env, environment);
        for statement in statements.iter() {
            match self.accept_statement(statement) {
                Ok(literal) => match literal {
                    TokenLiteral::LOX_NULL => (),
                    // Exit block early on reaching return
                    _ => {
                        self.curr_env = previous;
                        // Wrap return value in an error to propagate up to nearest function call
                        return Err(InterpreterError::Return(literal));
                    }
                }
                Err(error) => {
                    self.curr_env = previous;
                    return Err(error);
                }
            }
        }
        self.curr_env = previous;

        // Block ends 'naturally' when no errors or inner-returns are reached
        Ok(TokenLiteral::LOX_NULL)
    }

    fn visit_class_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Class { name, methods, superclass } => {
                let superclass = match superclass {
                    None => Ok(None),
                    Some(expr) => match self.accept_expr(expr) {
                        Ok(TokenLiteral::LOX_CALLABLE(constructor)) => match constructor.deref() {
                            LoxCallable::ClassConstructor(class) => Ok(Some(Rc::clone(class))),
                            _ => {
                                let err_msg = String::from("Superclass must be a class");
                                Err(InterpreterError::OperatorError {line: name.line, err_msg})
                            }
                        }
                        _ => {
                            let err_msg = String::from("Superclass must be a class");
                            Err(InterpreterError::OperatorError {line: name.line, err_msg})
                        }
                    }
                }?;

                self.curr_env.define(name.lexeme.clone(), TokenLiteral::LOX_NULL);

                // let mut prev_env = None;
                if let Some(class) = &superclass {
                    self.curr_env = Rc::new(Environment::new(Some(Rc::clone(&self.curr_env))));
                    self.curr_env.define(String::from("super"), TokenLiteral::LOX_INSTANCE(Rc::new(LoxInstance::new(Rc::clone(class)))));
                }

                let mut class_methods = HashMap::new();
                for method in methods.iter() {
                    match method {
                        Function { ptr } => {
                            let name = ptr.name.lexeme.clone();

                            // A bunch of type-checking boilerplate
                            let function = Rc::clone(ptr);
                            let function = LoxFunction::new(Function { ptr: function },
                                                            Rc::clone(&self.curr_env),
                                                            &ptr.name.lexeme == "init");

                            class_methods.insert(name, Rc::new(function));
                        }
                        _ => {
                            let err_msg = String::from("Non-method objects found in class body");
                            return Err(InterpreterError::OperatorError { err_msg, line: name.line })
                        }
                    }
                }

                if superclass.is_some() {
                    self.curr_env = mem::take(&mut Rc::clone(self.curr_env.enclosing.as_ref().unwrap()));
                }


                let class = LoxCallable::ClassConstructor(Rc::new(LoxClass::new(name.lexeme.clone(), superclass, class_methods)));
                self.curr_env.assign(name, TokenLiteral::LOX_CALLABLE(Rc::new(class)))?;
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-class statement passed to class visitor")
        }
    }

    fn visit_expression_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Expression { expression } => {
                self.accept_expr(expression)?;
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-expression statement passed to expr visitor")
        }
    }

    fn visit_print_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Print { expression } => {
                let value = self.accept_expr(expression)?;
                println!("{}", value);
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-print statement passed to print visitor")
        }
    }

    fn visit_var_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Var { name, initializer } => {
                let value = self.accept_expr(initializer)?;
                self.curr_env.define(name.lexeme.clone(), value);
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-var statement passed to var visitor")
        }
    }

    fn visit_if_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            If { expression, then_branch, else_branch} => {
                match Interpreter::is_truthy(&self.accept_expr(expression)?) {
                    true => self.accept_statement(then_branch),
                    false => self.accept_statement(else_branch),
                }
            }
            _ => unreachable!("Non-if statement passed to if visitor")
        }
    }

    fn visit_while_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            While { expression, body } => {
                while Interpreter::is_truthy(&self.accept_expr(expression)?) {
                    self.accept_statement(body)?;
                }
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-while statement passed to while visitor")
        }
    }

    fn visit_function_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Function { ptr } => {
                let curr_env = self.curr_env.clone();
                let function_obj = Function { ptr: Rc::clone(ptr) };
                let function_obj = LoxFunction::new(function_obj, curr_env, false);
                let function = Rc::new(LoxCallable::UserFunction(Rc::new(function_obj)));
                self.curr_env.define(ptr.as_ref().name.lexeme.clone(), TokenLiteral::LOX_CALLABLE(function));
                Ok(TokenLiteral::LOX_NULL)
            }
            _ => unreachable!("Non-function statement passed to function visitor")
        }
    }

    fn visit_return_stmt(&mut self, stmt: &Stmt) -> Result<TokenLiteral, InterpreterError> {
        match stmt {
            Return { value, .. } => {
                let value = self.accept_expr(value)?;
                Err(InterpreterError::Return(value))
            }
            _ => unreachable!("Non-return statement passed to return visitor")
        }
    }

    fn visit_literal_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Literal { value } => Ok(value.clone()),
            _ => unreachable!("Non-literal expression passed to literal visitor")
        }
    }

    fn visit_logical_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Logical { left, operator, right } => {
                let left = self.accept_expr(left)?;
                match (Interpreter::is_truthy(&left), operator.token_type) {
                    // Short-circuit
                    (true, OR) | (false, AND) => Ok(left),
                    (_, _) => self.accept_expr(right)
                }
            }
            _ => unreachable!("Non-logical expression passed to logical visitor")
        }
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Grouping { expression } => self.accept_expr(expression),
            _ => unreachable!("Non-group expression passed to group visitor")
        }
    }

    fn visit_binary_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Binary { left, operator, right, } => {
                // Recursively evaluate operands until they are usable literals
                let left = self.accept_expr(left)?;
                let right = self.accept_expr(right)?;
                match (left, right) {
                    // Two numbers
                    (TokenLiteral::LOX_NUMBER(left), TokenLiteral::LOX_NUMBER(right)) => {
                        match operator.token_type {
                            // Arithmetic
                            PLUS => Ok(TokenLiteral::LOX_NUMBER(left + right)),
                            MINUS => Ok(TokenLiteral::LOX_NUMBER(left - right)),
                            STAR => Ok(TokenLiteral::LOX_NUMBER(left * right)),
                            SLASH => Ok(TokenLiteral::LOX_NUMBER(left / right)),
                            // Logical
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_NUMBER(left);
                                let right = TokenLiteral::LOX_NUMBER(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_NUMBER(left);
                                let right = TokenLiteral::LOX_NUMBER(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            GREATER => Ok(TokenLiteral::LOX_BOOL(left > right)),
                            GREATER_EQUAL => Ok(TokenLiteral::LOX_BOOL(left >= right)),
                            LESS => Ok(TokenLiteral::LOX_BOOL(left < right)),
                            LESS_EQUAL => Ok(TokenLiteral::LOX_BOOL(left <= right)),
                            _ => {
                                let err_msg = String::from("Unrecognized operator passed between two numbers");
                                Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                            }
                        }
                    }
                    // Two strings
                    (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => {
                        match operator.token_type {
                            PLUS => Ok(TokenLiteral::LOX_STRING(Rc::new(format!("{left}{right}")))),
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_STRING(left);
                                let right = TokenLiteral::LOX_STRING(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            _ => {
                                let err_msg = String::from("Non-concatenating operator passed between two strings");
                                Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                            }
                        }
                    }
                    // Two bools
                    (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => {
                        match operator.token_type {
                            EQUAL_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(left, right)))
                            }
                            BANG_EQUAL => {
                                let left = TokenLiteral::LOX_BOOL(left);
                                let right = TokenLiteral::LOX_BOOL(right);
                                Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(left, right)))
                            }
                            _ => {
                                let err_msg = String::from("Non-equality operators passed between two bools");
                                Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                            }
                        }
                    }
                    // Two nils
                    (TokenLiteral::LOX_NULL, TokenLiteral::LOX_NULL) => match operator.token_type {
                        EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(Interpreter::is_equal(
                            TokenLiteral::LOX_NULL,
                            TokenLiteral::LOX_NULL,
                        ))),
                        BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_equal(
                            TokenLiteral::LOX_NULL,
                            TokenLiteral::LOX_NULL,
                        ))),
                        _ => {
                            let err_msg = String::from("Non-equality operators passed between two nils");
                            Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                        }
                    },
                    (TokenLiteral::LOX_CALLABLE(left), TokenLiteral::LOX_CALLABLE(right)) => {
                        match operator.token_type {
                            EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(Rc::ptr_eq(&left, &right))),
                            BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(!Rc::ptr_eq(&left, &right))),
                            _ => {
                                let err_msg = String::from("Non-equality operators passed between two function pointers");
                                Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                            }
                        }
                    },
                    (TokenLiteral::LOX_INSTANCE(left), TokenLiteral::LOX_INSTANCE(right)) => {
                        match operator.token_type {
                            EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(Rc::ptr_eq(&left, &right))),
                            BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(!Rc::ptr_eq(&left, &right))),
                            _ => {
                                let err_msg = String::from("Non-equality operators passed between two class instances");
                                Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                            }
                        }
                    }
                    // Operands of arbitrary, non-equal types
                    (_, _) => match operator.token_type {
                        EQUAL_EQUAL => Ok(TokenLiteral::LOX_BOOL(false)),
                        BANG_EQUAL => Ok(TokenLiteral::LOX_BOOL(true)),
                        _ => {
                            let err_msg = String::from("Mismatched types operated on");
                            Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                        }
                    },
                }
            }
            _ => unreachable!("Non-binary expression passed to binary visitor")
        }
    }

    fn visit_call_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Call { callee, paren, arguments } => {
                let callee = self.accept_expr(callee)?;
                let mut parameters = Vec::with_capacity(arguments.len());
                for arg in arguments.iter() {
                    parameters.push(self.accept_expr(arg)?)
                }

                match callee {
                    TokenLiteral::LOX_CALLABLE(callable) => {
                        match callable.arity() == parameters.len() {
                            true => {
                                if let LoxCallable::ClassConstructor(_) = *callable {
                                    // Add class instance as last parameter
                                    parameters.push(TokenLiteral::LOX_CALLABLE(Rc::clone(&callable)));
                                }
                                callable.call(self, parameters)
                            },
                            false => {
                                let err_msg = format!("Expected {} arguments but got {}.", callable.arity(), parameters.len());
                                Err(InterpreterError::OperatorError { line: paren.line, err_msg})
                            }
                        }
                    }
                    _ => {
                        let err_msg = String::from("Can only call functions and class instances");
                        Err(InterpreterError::OperatorError { line: paren.line, err_msg})
                    }
                }
            }
            _ => unreachable!("Non-call expression passed to call visitor")
        }
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Unary { operator, right } => {
                let right = self.accept_expr(right)?;
                match operator.token_type {
                    MINUS => match right {
                        TokenLiteral::LOX_NUMBER(num) => Ok(TokenLiteral::LOX_NUMBER(-num)),
                        _ => {
                            let err_msg = String::from("Minus operator used on non-numerical operand");
                            Err(InterpreterError::OperatorError { line: operator.line, err_msg })
                        }
                    },
                    BANG => Ok(TokenLiteral::LOX_BOOL(!Interpreter::is_truthy(&right))),
                    _ => unreachable!("Only two unary operators exist")
                }
            }
            _ => unreachable!("Non-unary expression passed to unary visitor")
        }
    }

    fn visit_variable_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        self.lookup_variable(expr)
    }

    fn lookup_variable(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Variable { name, id } | This { name, id } => {
                match self.locals.get(id) {
                    Some(distance) => self.curr_env.deref().get_at(*distance, name),
                    None => self.global_env.deref().get(name)
                }
            }
            _ => unreachable!("Non-variable expression passed to variable lookup")
        }
    }

    fn visit_assign_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        self.assign_variable(expr)
    }

    fn assign_variable(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Assign { name, value , id} => {
                let value = self.accept_expr(value)?;

                match self.locals.get(id) {
                    Some(distance) => self.curr_env.deref().assign_at(*distance, name, value.clone()),
                    None => self.global_env.deref().assign(name, value.clone()),
                }?;

                Ok(value)
            }
            _ => unreachable!("Non-assignment expression passed to assignment visitor")
        }
    }

    fn visit_get_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Get { object, name , .. } => {
                let object = self.accept_expr(object)?;
                match object {
                    TokenLiteral::LOX_INSTANCE(instance) => instance.get(Rc::clone(&instance), name),
                    _ => {
                        let err_msg = String::from("Only instances have properties.");
                        Err(InterpreterError::OperatorError { err_msg, line: name.line})
                    }
                }
            },
            _ => unreachable!("Non-get expression passed to get visitor")
        }
    }

    fn visit_set_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        match expr {
            Set { object, name , value, .. } => {
                let object = self.accept_expr(object)?;
                match object {
                    TokenLiteral::LOX_INSTANCE(instance) => {
                        let value = self.accept_expr(value)?;
                        instance.set(name, value.clone());
                        Ok(value)
                    }
                    _ => {
                        let err_msg = String::from("Only instances have fields.");
                        Err(InterpreterError::OperatorError { err_msg, line: name.line})
                    }
                }
            },
            _ => unreachable!("Non-get expression passed to get visitor")
        }
    }

    fn visit_super_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        // This is by far the most spaghetti piece of code I've ever written

        let Super { keyword, id, method } = expr else {
            unreachable!("Non-super expression passed to super visitor")
        };
        let distance = self.locals.get(id).unwrap();
        let superclass = self.curr_env.get_at(*distance, keyword)?;
        let TokenLiteral::LOX_INSTANCE(superclass ) = superclass else {
            unreachable!("'super' maps to Lox_Callable token literals")
        };

        let dummy_this = Token { token_type: NIL, line: -1, lexeme: String::from("this"), literal: TokenLiteral::LOX_NULL};
        let TokenLiteral::LOX_INSTANCE(instance) = self.curr_env.get_at(*distance - 1, &dummy_this)? else {
            unreachable!()
        };

        let super_method = superclass.class.find_method(&method.lexeme);
        if super_method.is_none() {
            let err_msg = format!("Undefined property '{}'", method.lexeme);
            return Err(InterpreterError::OperatorError {line: method.line, err_msg});
        }
        Ok(TokenLiteral::LOX_CALLABLE(Rc::new(LoxCallable::UserFunction(Rc::new(super_method.unwrap().bind(instance))))))
    }

    fn visit_this_expr(&mut self, expr: &Expr) -> Result<TokenLiteral, InterpreterError> {
        self.lookup_variable(expr)
    }

    fn is_truthy(literal: &TokenLiteral) -> bool {
        match literal {
            TokenLiteral::LOX_BOOL(bool_value) => *bool_value,
            TokenLiteral::LOX_NULL => false,
            _ => true,
        }
    }

    fn is_equal(left: TokenLiteral, right: TokenLiteral) -> bool {
        match (left, right) {
            (TokenLiteral::LOX_NUMBER(left), TokenLiteral::LOX_NUMBER(right)) => left == right,
            (TokenLiteral::LOX_STRING(left), TokenLiteral::LOX_STRING(right)) => left == right,
            (TokenLiteral::LOX_BOOL(left), TokenLiteral::LOX_BOOL(right)) => left == right,
            (TokenLiteral::LOX_NULL, TokenLiteral::LOX_NULL) => true,
            (_, _) => false,
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        match expr {
            Variable { id, .. } | Assign { id, .. } | This { id, .. } | Super { id, .. }=> {
                self.locals.insert(*id, depth);
            }
            _ => unreachable!("Non-local variable accessing statement passed to local resolver")
        }
    }
}
