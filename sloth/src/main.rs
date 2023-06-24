#![feature(let_chains)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod analysis;
pub mod lexer;
pub mod parser;
pub mod sloth_std;
pub mod symtable;

use std::{env, fs};

use analysis::analyze;
use itertools::Itertools;
use lexer::Lexer;
use parser::AstParser;
use symtable::{Symbol, SymbolTable, SymbolType};

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
    global_symtable.insert("print".to_owned(), Symbol::new(SymbolType::Function));
    global_symtable.insert("println".to_owned(), Symbol::new(SymbolType::Function));
    global_symtable.insert("readln".to_owned(), Symbol::new(SymbolType::Function));

    // Parsing
    let tokens = Lexer::new(&source).collect_vec();
    let mut ast = AstParser::parse(tokens, global_symtable).unwrap();

    if let Err(error) = analyze(&mut ast) {
        eprintln!("Error on line {}: {error}", error.line() + 1);
        return;
    }

    println!("{ast:#?}");

    // let graph = GraphBuilder::generate(&ast).unwrap();
    // println!("{graph}");
}
