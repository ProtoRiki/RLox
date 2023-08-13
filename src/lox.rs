use std::io::{self, Write};
use std::fs;
use std::process;

use crate::token::Token;
use crate::token_type::TokenType;
use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::interpreter::{Interpreter, InterpreterError};
use crate::resolver::Resolver;

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

pub fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(file_str) => {
            let mut interpreter = Interpreter::new();
            run(&mut interpreter, file_str)
        },
        Err(err) => {
            eprintln!("{err}");
            unsafe {
                HAD_ERROR = true;
            }
        }
    }
    unsafe {
        if HAD_ERROR {
            process::exit(65);
        }
        if HAD_RUNTIME_ERROR {
            process::exit(70);
        }
    }
}

pub fn run_prompt() {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 { break; }
                run(&mut interpreter, buffer);
                unsafe { HAD_ERROR = false; }
            },
            Err(err) => {
                eprintln!("{err}");
                process::exit(65);
            },
        }
    }
}

pub fn run(interpreter: &mut Interpreter, source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    // Return early if parsing fails
    if statements.is_err() { return; }

    let statements = statements.unwrap();
    let mut resolver = Resolver::new(interpreter);
    resolver.resolve_statements(&statements);

    if unsafe { HAD_ERROR } { return; }

    interpreter.interpret(&statements);
}

pub fn error(line: i32, message: &str) {
    report(line, "", message);
}

pub fn token_error(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(token.line, "at end", message);
    }
    else {
        report(token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

pub fn runtime_error(error: &InterpreterError) {
    match error {
        InterpreterError::OperatorError { line, err_msg } => {
            eprintln!("[line {}] Runtime Error {}", line, err_msg);
        }
    }
    unsafe { HAD_RUNTIME_ERROR = true }

}

pub fn report(line: i32, loc: &str, message: &str) {
    eprintln!("[line {line}] Syntax Error: {loc}: {message}");
    unsafe { HAD_ERROR = true; }
}