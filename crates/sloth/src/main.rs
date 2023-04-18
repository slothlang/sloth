#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod compiler;
pub mod lexer;
pub mod parser;

use std::collections::HashMap;

// use std::{env, fs};
use itertools::Itertools;
use lexer::Lexer;
use parser::AstParser;
use sloth_vm::value::Function;
use sloth_vm::{ObjectMap, VM};

use crate::compiler::Compiler;

fn main() {
    // let args = env::args().collect_vec();
    //
    // if args.len() < 2 {
    //     println!("Sloth programming language interpreter\n");
    //     println!("Usage: sloth <file>");
    //     return;
    // }
    //
    // let source_path = &args[1];
    // let Ok(source) = fs::read_to_string(source_path) else {
    //     println!("Error while reading '{source_path}'");
    //     return;
    // };
    // let source = " 3 + 7 ;";
    // let source = r#"

    //     fn hello() -> int {
    //         return 3 + 7;
    //     }

    //     hello();
    //     hello();
    //     hello();
    //     hello();
    //     hello();
    //     hello();

    // "#;
    let source = r#"

        fn hello() -> int {
            var x = 5;
            x = 7;
            return x;
        }

        hello();

    "#;

    let tokens = Lexer::new(source).collect_vec();
    let ast = AstParser::new(tokens).parse();
    let mut object_map = ObjectMap::default();
    let code = Compiler::compile(&mut object_map, HashMap::default(), ast.clone());

    println!("{ast:?}\n\n");
    println!("{:?}\n\n", code.constants);
    println!("{:?}\n\n", code.code);

    let mut vm = VM::new(object_map, Function::root(code));
    vm.run();
    println!("{:?}", vm.stack.peek());
}
