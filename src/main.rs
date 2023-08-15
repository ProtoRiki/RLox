mod token_type;
mod token;
mod scanner;
mod token_literal;
mod lox;
mod parser;
mod expression;
mod interpreter;
mod statement;
mod environment;
mod callable;
mod function;
mod clock;
mod function_object;
mod native;
mod resolver;
mod class;
mod class_instance;

use std::env;
use std::cmp::Ordering;
use std::process;

use lox::{run_file, run_prompt};

const ARGS_LIMIT: usize = 2;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&ARGS_LIMIT) {
        Ordering::Greater => {
            println!("usage: rlox script");
            process::exit(64);
        },
        Ordering::Equal => run_file(&args[1]),
        Ordering::Less => run_prompt(),
    }
}

