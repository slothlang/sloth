#![feature(test, let_chains)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod ast;
pub mod lexer;

use ast::{Expression, Operation, Value};
use lexer::Lexer;

use crate::ast::Statement;

const SOURCE: &str = r#"

val variable = 5;

if variable <= 7 {
    print "Hello World";
}

"#;

fn main() {
    let lexer = Lexer::new(SOURCE);
    for token in lexer {
        print!("{} ", token.lexeme);
    }

    println!("-------");

    let a = Expression::Literal(Value(7));
    let b = Expression::Binary {
        operation: Operation::Add,
        lhs: &Expression::Literal(Value(5)),
        rhs: &a,
    };

    let stmt = Statement::Expression { expr: &b };

    println!("{stmt}");
}
