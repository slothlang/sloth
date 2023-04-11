use crate::lexer::{Token, TokenType};
#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Con,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,

    BWSftRight,
    BWSftLeft,
    BWAnd,
    BWOr,
    BWXor,

    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    NotEq,
    LogAnd,
    LogOr,
}
#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,

    BWComp,
}
#[derive(Debug, PartialEq)]
pub enum Literal {
    Integer(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Regex(String),
    List(Vec<Expr>), // TODO: holy shit we forgor listys
}
#[derive(Debug, PartialEq)]
pub enum Expr {
    Grouping(Box<Expr>),
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
        ident: Box<Expr>,
        args: Vec<Expr>,
    },
    Variable(String),
    Literal(Literal),
    Lambda, // TODO: Lambda bitch
}

pub struct FuncArgs {
    pub name: String,
    pub typ: Option<String>,
}

pub enum Stmt {
    ExprStmt(Expr),
    DefineFunction {
        ident: String,
        args: Vec<FuncArgs>,
        body: Vec<Stmt>,
        return_type: Option<String>,
    },
    DefineVariable {
        name: String,
        value: Expr,
        typ: Option<String>,
    },
    DefineValue {
        name: String,
        value: Expr,
        typ: Option<String>,
    },
    If {
        expr: Expr,
        body: Vec<Stmt>,
        else_if: Vec<(Expr, Stmt)>,
        els: Option<Box<Stmt>>,
    },
    For {
        name: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}

pub struct AstParser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

/// Implementation containing utilities used by the parsers internal components
impl<'a> AstParser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, index: 0 }
    }
    pub fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn advance(&mut self) -> Option<&Token> {
        if self.eof() {
            return None;
        }

        self.index += 1;
        Some(&self.tokens[self.index - 1])
    }

    pub fn advance_if(&mut self, next: impl FnOnce(&Token) -> bool) -> bool {
        if self.eof() {
            return false;
        }

        if next(self.peek()) {
            self.advance();
            return true;
        }

        false
    }

    pub fn advance_if_eq(&mut self, next: &TokenType) -> bool {
        self.advance_if(|it| it.tt == *next)
    }

    pub fn consume(&mut self, next: TokenType, error: &str) {
        if std::mem::discriminant(&self.peek().tt) != std::mem::discriminant(&next) {
            panic!("{error}");
        }
        self.advance();
    }

    pub fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}
