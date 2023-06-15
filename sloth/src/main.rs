#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod symbol;

use std::{env, fs};

use compiler::Compiler;
use itertools::Itertools;
use lexer::Lexer;
use parser::graph::GraphBuilder;
use parser::AstParser;

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

    let tokens = Lexer::new(&source).collect_vec();
    let ast = AstParser::parse(tokens).unwrap();

    let graph = GraphBuilder::generate(&ast).unwrap();
    println!("{graph}");

    // Compiler::compile(ast).unwrap();

    // let context = Context::create();
    // let compiler = Compiler::new(&context);
    //
    // compiler.compile(ast);
}