#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod ast;
pub mod lexer;

use lexer::Lexer;

const SOURCE: &str = r#"

val variable = 5;

if variable >= 7 {
    print "Hello World";
}

"#;

fn main() {
    let lexer = Lexer::new(SOURCE);
    for token in lexer {
        print!("({}) ", token.lexeme);
    }
}
