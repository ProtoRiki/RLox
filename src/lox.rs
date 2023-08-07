use std::io;
use std::io::Write;
use std::fs;
use std::process;


use crate::token::Token;
use crate::token_type::TokenType::EOF;
use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::interpreter::{Interpreter, InterpreterError};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;
static INTERPRETER: Interpreter = Interpreter{};
// Make interpreter static so that successive calls to run()
// inside a REPL session reuse the same interpreter.

pub fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(file_str) => run(file_str),
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
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                run(buffer);
                unsafe {
                    HAD_ERROR = false;
                }
            },
            Err(err) => {
                eprintln!("{err}");
                process::exit(65);
            },
        }
    }
}

pub fn run(source: String) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    let expression = parser.parse();
    if expression.is_err() {
        return;
    }
    let expression = expression.unwrap();
    INTERPRETER.interpret(expression);
}

pub fn error(line: i32, message: &str) {
    report(line, "", message);
}

pub fn token_error(token: &Token, message: &str) {
    if token.token_type == EOF {
        report(token.line, "at end", message);
    }
    else {
        report(token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

pub fn runtime_error(error: &InterpreterError) {
    match error {
        InterpreterError::LiteralError(msg) => eprintln!("{}", msg),
        InterpreterError::OperatorError { operator, msg } => {
            eprintln!("{}\n[line {}]", msg, operator.line);
        }
    }
    unsafe { HAD_RUNTIME_ERROR = true }

}

pub fn report(line: i32, loc: &str, message: &str) {
    eprintln!("[line {line}] Lexing Error {loc}: {message}");
    unsafe { HAD_ERROR = true; }
}