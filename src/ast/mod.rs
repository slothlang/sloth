#![allow(dead_code)]

pub mod display;

use crate::lexer::Token;

#[derive(Clone)]
pub enum Statement<'a> {
    Val {
        identifier: &'a Token<'a>,
        initializer: &'a Expression<'a>,
    },
    Var {
        identifier: &'a Token<'a>,
        initializer: &'a Expression<'a>,
    },
    Expression {
        expr: &'a Expression<'a>,
    },
}

#[derive(Clone)]
pub enum Expression<'a> {
    // Basic
    Literal(Value),
    Unary {
        operation: Operation,
        expr: &'a Expression<'a>,
    },
    Binary {
        operation: Operation,
        lhs: &'a Expression<'a>,
        rhs: &'a Expression<'a>,
    },
    // Grouping
}

#[derive(Clone)]
pub enum Operation {
    Add,
    Subtract,
}

#[derive(Clone)]
pub struct Value(pub i32);

#[test]
fn test() {
    let right = Expression::Literal(Value(7));
    let _ = Expression::Binary {
        operation: Operation::Add,
        lhs: &Expression::Literal(Value(5)),
        rhs: &right,
    };
}
