#![feature(let_chains)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod analysis;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod sloth_std;
pub mod symtable;

use std::fs::File;
use std::{env, fs};

use codegen::Compiler;
use inkwell::context::Context;
use inkwell::targets::FileType;
use itertools::Itertools;
use lexer::Lexer;
use parser::AstParser;
use symtable::{Symbol, SymbolTable};

use crate::analysis::analyze;
use crate::parser::graph::GraphBuilder;
use crate::symtable::Type;

fn main() {
    let args = env::args().collect_vec();

    if args.len() < 2 {
        println!("Sloth programming language interpreter\n");
        println!("Usage: sloth <file>");
        return;
    }

    let source_path = &args[1];
    let Ok(source) = fs::read_to_string(source_path) else {
        eprintln!("Error while reading '{source_path}'");
        return;
    };

    // Symbol table
    let mut global_symtable = SymbolTable::new();
    global_symtable.insert("Void".into(), Symbol::Type(Type::Void));
    global_symtable.insert("Int".into(), Symbol::Type(Type::Integer));
    global_symtable.insert("Float".into(), Symbol::Type(Type::Float));
    global_symtable.insert("Bool".into(), Symbol::Type(Type::Boolean));

    // Parsing
    let tokens = Lexer::new(&source).collect_vec();
    let mut ast = AstParser::parse(tokens, global_symtable).unwrap();

    if let Err(error) = analyze(&mut ast) {
        eprintln!("Error on line {}: {error}", error.line() + 1);
        return;
    }

    // println!("{ast:#?}");
    // println!("Suces");

    let context = Context::create();
    let mut compiler = Compiler::new(&context, "s");
    let mut output_file = File::create("output.o").unwrap();

    compiler.codegen(&ast);
    compiler.write_obj(&mut output_file, FileType::Object);

    // Compiler::codegen(&context, "hi", &ast);

    // let graph = GraphBuilder::generate(Some(&source), &ast).unwrap();
    // println!("{graph}");
}
