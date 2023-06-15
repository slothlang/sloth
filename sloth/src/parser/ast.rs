use std::fmt::{format, Display};

use crate::lexer::{self, TokenType};
use crate::parser::ParsingError;

#[derive(PartialEq, Clone, Debug)]
pub struct Expr {
    pub id: i32,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(id: i32, kind: ExprKind) -> Self {
        Self { id, kind }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Grouping(Box<Expr>),
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
}

#[derive(PartialEq, Clone, Debug)]
pub struct Stmt {
    pub id: i32,
    pub kind: StmtKind,
}

impl Stmt {
    pub fn new(id: i32, kind: StmtKind) -> Self {
        Self { id, kind }
    }
}

// TODO: For loops
// TODO: Values & Constants
#[derive(PartialEq, Clone, Debug)]
pub enum StmtKind {
    Block(Vec<Stmt>),
    ExprStmt(Expr),
    IfStmt {
        condition: Expr,
        if_then: Box<Stmt>,
        else_then: Option<Box<Stmt>>,
    },
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,
    },
    DefineVariable {
        identifier: String,
        value: Expr,
        typ: String,
    },
    AssignVariable {
        identifier: String,
        value: Expr,
    },
    /// A function definition. Output is None when the function returns nothing
    /// meaning void, otherwise it is the name of the type the function
    /// returns.
    DefineFunction {
        identifier: String,
        inputs: Vec<FunctionInput>,
        output: Option<String>,
        body: Box<Stmt>,
    },
    Return(Expr),
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionInput {
    pub identifier: String,
    pub typ: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Array(Vec<ExprKind>),
}

impl From<lexer::Literal> for Literal {
    fn from(value: lexer::Literal) -> Self {
        use lexer::Literal;

        match value {
            Literal::Integer(value) => Self::Integer(value),
            Literal::Float(value) => Self::Float(value),
            Literal::Boolean(value) => Self::Boolean(value),
            Literal::Character(value) => Self::Character(value),
            Literal::String(value) => Self::String(value),
        }
    }
}

impl From<Literal> for ExprKind {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Literal::Integer(i) => format!("{i}"),
            Literal::Float(f) => format!("{f}"),
            Literal::Boolean(b) => format!("{b}"),
            Literal::Character(c) => format!("'{c}'"),
            Literal::String(s) => format!("\"{s}\""),
            Literal::Array(a) => format!("<Array>"),
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Con,
    Sub,
    Mul,
    Div,
    Mod,

    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    NotEq,

    LogicalAnd,
    LogicalOr,

    Range,
}

impl TryFrom<TokenType> for BinaryOp {
    type Error = ParsingError;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        let operation = match value {
            TokenType::Plus => Self::Add,
            TokenType::PlusPlus => Self::Con,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
            TokenType::Perc => Self::Mod,

            TokenType::Lt => Self::Lt,
            TokenType::Gt => Self::Gt,
            TokenType::LtEq => Self::LtEq,
            TokenType::GtEq => Self::GtEq,
            TokenType::EqEq => Self::EqEq,
            TokenType::BangEq => Self::NotEq,

            TokenType::AmpAmp => Self::LogicalAnd,
            TokenType::PipePipe => Self::LogicalOr,

            TokenType::DotDot => Self::Range,

            _ => return Err(ParsingError::InvalidOp),
        };

        Ok(operation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl TryFrom<TokenType> for UnaryOp {
    type Error = ParsingError;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        let operation = match value {
            TokenType::Bang => Self::Not,
            TokenType::Minus => Self::Neg,

            _ => return Err(ParsingError::InvalidOp),
        };

        Ok(operation)
    }
}
