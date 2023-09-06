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
pub mod symtable;

use std::fs::File;
use std::{env, fs};

use codegen::Codegen;
use inkwell::context::Context;
use inkwell::targets::FileType;
use itertools::Itertools;
use lexer::Lexer;
use parser::AstParser;
use symtable::{Symbol, SymbolTable, ValueSymbol};

use crate::analysis::analyze;
use crate::symtable::Type;

fn main() {
    let args = env::args().collect_vec();

    if args.len() < 2 {
        println!("Sloth programming language interpreter\n");
        println!("Usage: sloth <file...>");
        return;
    }

    // Reading source files
    let mut source = String::new();
    for path in args.iter().skip(1) {
        let Ok(contents) = fs::read_to_string(path) else {
            eprintln!("Error while reading '{path}'");
            return;
        };
        source.push_str(&contents);
        let len = contents.lines().collect_vec().len();
        let padding = 1000 - len;
        for _ in 0..padding {
            source.push('\n');
        }
    }

    // Parsing
    let tokens = Lexer::new(&source).collect_vec();
    let global_symtable = mk_symtable();

    let mut ast = match AstParser::parse(tokens, global_symtable) {
        Ok(node) => node,
        Err(error) => {
            eprintln!(
                "Error in file {} on line {}: {error}",
                args[1 + (error.line() / 1_000) as usize],
                error.line() % 1000 + 1,
            );
            return;
        }
    };

    if let Err(error) = analyze(&mut ast) {
        eprintln!(
            "Error in file {} on line {}: {error}",
            args[1 + (error.line() / 1_000) as usize],
            error.line() % 1000 + 1,
        );
        return;
    }

    // Generating code for module
    let context = Context::create();
    let mut codegen = Codegen::new(&context, "s");
    let mut output_file = File::create("output.o").unwrap();

    codegen.codegen(&ast);
    codegen.write_obj(&mut output_file, FileType::Object);
}

fn mk_symtable() -> SymbolTable {
    // Symbol table
    let mut global_symtable = SymbolTable::new();
    global_symtable.insert("Void".into(), Symbol::Type(Type::Void));
    global_symtable.insert("Int".into(), Symbol::Type(Type::Integer));
    global_symtable.insert("Float".into(), Symbol::Type(Type::Float));
    global_symtable.insert("Bool".into(), Symbol::Type(Type::Boolean));
    global_symtable.insert("String".into(), Symbol::Type(Type::String));

    // Inputs aren't type checked but outputs are
    let dummyi = Symbol::Value(ValueSymbol {
        typ: Type::Function {
            inputs: vec![],
            output: Box::new(Type::Integer),
        },
        id: 0,
        mutable: true,
    });

    let dummyf = Symbol::Value(ValueSymbol {
        typ: Type::Function {
            inputs: vec![],
            output: Box::new(Type::Float),
        },
        id: 0,
        mutable: true,
    });

    let dummyb = Symbol::Value(ValueSymbol {
        typ: Type::Function {
            inputs: vec![],
            output: Box::new(Type::Boolean),
        },
        id: 0,
        mutable: true,
    });

    let dummys = Symbol::Value(ValueSymbol {
        typ: Type::Function {
            inputs: vec![],
            output: Box::new(Type::Boolean),
        },
        id: 0,
        mutable: true,
    });

    global_symtable.insert("vlen".into(), dummyi.clone());

    global_symtable.insert("vpushi".into(), dummyi.clone());
    global_symtable.insert("vpushf".into(), dummyf.clone());
    global_symtable.insert("vpushb".into(), dummyb.clone());
    global_symtable.insert("vpushs".into(), dummys.clone());

    global_symtable.insert("vpopi".into(), dummyi.clone());
    global_symtable.insert("vpopf".into(), dummyf.clone());
    global_symtable.insert("vpopb".into(), dummyb.clone());
    global_symtable.insert("vpops".into(), dummys.clone());

    global_symtable.insert("vgeti".into(), dummyi.clone());
    global_symtable.insert("vgetf".into(), dummyf.clone());
    global_symtable.insert("vgetb".into(), dummyb.clone());
    global_symtable.insert("vgets".into(), dummys.clone());

    global_symtable.insert("vseti".into(), dummyi);
    global_symtable.insert("vsetf".into(), dummyf);
    global_symtable.insert("vsetb".into(), dummyb);
    global_symtable.insert("vsets".into(), dummys);

    global_symtable
}
