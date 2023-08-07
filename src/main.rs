mod token_type;
mod token;
mod scanner;
mod token_literal;
mod lox;
mod parser;
mod expression;
mod interpreter;
mod statement;

use std::env;
use std::process;
use lox::{run_file, run_prompt};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("usage: rlox script");
        process::exit(64);
    }
    else if args.len() == 2 {
        run_file(&args[1]);
    }
    else {
        run_prompt();
    }
}

