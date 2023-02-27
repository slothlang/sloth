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

use itertools::Itertools;

use crate::ast::parser::AstParser;
use crate::ast::AstVisitor;
use crate::interpreter::AstInterpreter;
use crate::lexer::Lexer;

const SOURCE: &str = r#"

val variable = 5 + 6 * 2;

if variable == 17 {
    print "Hello World";
}

fn fib(n: i32) -> i32 {
    if n == 0 || n == 1 {
        return n;
    }

    var grandparent = 0;
    var parent = 1;
    var me = 0;

    for i in 0..n-1 {
        me          = parent + grandparent;
        grandparent = parent;
        parent      = me;
    }

    return me;
}

print fib(5);

"#;

fn main() {
    let lexer = Lexer::new("for x in 0..5 {}");
    let tokens = lexer.collect_vec();
    let mut parser = AstParser::new(tokens);
    let ast = parser.parse();

    println!("{ast:#?}");
    println!("--- Program Output ---");

    let mut interpreter = AstInterpreter::default();
    interpreter.interpret(&ast);
}
