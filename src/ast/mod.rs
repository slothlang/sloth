use crate::lexer::{Literal, TokenType};

pub mod parser;
pub mod printer;

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Val {
        ident: String,
        value: Expr,
    },
    Var {
        ident: String,
        value: Expr,
    },
    Assignment {
        ident: String,
        value: Expr,
    },
    Function {
        ident: String,
        arguments: Vec<FunctionArgument>,
        return_type: String,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        binding: String,
        range: (Expr, Expr),
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Return {
        value: Expr,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct FunctionArgument {
    name: String,
    types: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    Grouping(Box<Expr>),
    Call {
        ident: String,
        arguments: Vec<Expr>,
    },
    Binary {
        operator: TokenType,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        operator: TokenType,
        expr: Box<Expr>,
    },
}

pub trait AstVisitor<T = ()> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
    fn visit_expr(&mut self, expr: &Expr) -> T;
}
