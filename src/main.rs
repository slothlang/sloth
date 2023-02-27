#![feature(test, let_chains)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]
#![allow(unused)]

pub mod ast;
pub mod interpreter;
pub mod lexer;

use std::io::{self, BufRead, Read, Write};
use std::{env, fs};

use itertools::Itertools;
use rand::Rng;

use crate::ast::parser::AstParser;
use crate::ast::AstVisitor;
use crate::interpreter::{AstInterpreter, InternalFunction, Value};
use crate::lexer::Lexer;

fn main() {
    let args = env::args().collect_vec();

    if args.len() < 2 {
        println!("Sloth programming language interpreter\n");
        println!("Usage: sloth <file>");
        return;
    }

    let source_path = &args[1];
    let Ok(source) = fs::read_to_string(source_path) else {
        println!("Error while reading '{source_path}'");
        return;
    };

    let lexer = Lexer::new(&source);
    let tokens = lexer.collect_vec();
    let mut parser = AstParser::new(tokens);
    let ast = parser.parse();

    println!("--- Abstract Syntax Tree ---");
    println!("{ast:#?}");

    println!("--- Program Output ---");

    let mut interpreter = AstInterpreter::default();

    // Defining some builtin callables for our interpreter
    interpreter.callables.insert(
        "print".to_owned(),
        Box::new(InternalFunction(&|args| {
            use std::fmt::Write;

            let mut buffer = String::new();
            for arg in args {
                write!(&mut buffer, "{}", arg);
            }

            let mut stdout = io::stdout();
            stdout.lock().write_all(buffer.as_bytes());
            stdout.flush();

            Value::Nil
        })),
    );

    interpreter.callables.insert(
        "println".to_owned(),
        Box::new(InternalFunction(&|args| {
            use std::fmt::Write;

            let mut buffer = String::new();
            for arg in args {
                write!(&mut buffer, "{}", arg);
            }
            writeln!(&mut buffer);

            let mut stdout = io::stdout();
            stdout.lock().write_all(buffer.as_bytes());
            stdout.flush();

            Value::Nil
        })),
    );

    interpreter.callables.insert(
        "readln".to_owned(),
        Box::new(InternalFunction(&|args| {
            let stdin = io::stdin();
            let mut line = String::new();
            stdin
                .lock()
                .read_line(&mut line)
                .expect("Failed to read line from stdin");
            line.pop();

            Value::String(line)
        })),
    );

    interpreter.callables.insert(
        "random".to_owned(),
        Box::new(InternalFunction(&|args| {
            let result = match args {
                [] => rand::thread_rng().gen_range(1..=100),
                [Value::Number(max)] => rand::thread_rng().gen_range(0..=*max),
                [Value::Number(min), Value::Number(max)] => {
                    rand::thread_rng().gen_range(*min..=*max)
                }
                _ => panic!("Invalid usage of 'random' function"),
            };

            Value::Number(result)
        })),
    );

    interpreter.callables.insert(
        "len".to_owned(),
        Box::new(InternalFunction(&|args| {
            let result = match &args[0] {
                Value::String(value) => value.len() as i32,
                _ => panic!("Invalid usage of 'len' function"),
            };

            Value::Number(result)
        })),
    );

    interpreter.callables.insert(
        "parse_int".to_owned(),
        Box::new(InternalFunction(&|args| {
            let result = match &args[0] {
                Value::String(value) => value.parse::<i32>(),
                _ => panic!("Invalid usage of 'parse_int' function"),
            }
            .expect("Provided string was not an intenger");

            Value::Number(result)
        })),
    );

    interpreter.interpret(&ast);
}
