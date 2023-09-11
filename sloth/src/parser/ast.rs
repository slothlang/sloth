use std::fmt::Display;

use crate::lexer::{self, TokenType};
use crate::parser::ParsingError;
use crate::symtable::{SymbolTable, Type};

#[derive(PartialEq, Clone, Debug)]
/// AstNode that is either an Expr or Stmt, typically used for iterating over an
/// Ast for analysis reasons.
pub enum AstNode<'a> {
    Expr(&'a Expr),
    Stmt(&'a Stmt),
}

impl<'a> AstNode<'a> {
    pub fn children(&self) -> impl Iterator<Item = AstNode> {
        let mut children = Vec::new();
        match self {
            Self::Expr(expr) => children.extend(expr.children()),
            Self::Stmt(stmt) => children.extend(stmt.children()),
        }
        children.into_iter()
    }

    pub fn line(&self) -> u32 {
        match self {
            Self::Expr(expr) => expr.line,
            Self::Stmt(stmt) => stmt.line,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub id: i32,
    pub line: u32,
    pub kind: ExprKind,
    pub symtable: SymbolTable,

    /// Type of the expression. If None it means the type hasn't yet been
    /// checked.
    pub typ: Option<Type>,
    pub is_const: bool,
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.kind == other.kind
    }
}

impl Expr {
    pub fn new(id: i32, line: u32, kind: ExprKind, symtable: SymbolTable) -> Self {
        /// Recursivly check if a expression is constant
        fn is_const(kind: &ExprKind) -> bool {
            match kind {
                ExprKind::Literal(_) => true,
                ExprKind::Grouping(child) => is_const(&child.kind),
                ExprKind::BinaryOp { lhs, rhs, .. } => is_const(&lhs.kind) && is_const(&rhs.kind),
                ExprKind::UnaryOp { value, .. } => is_const(&value.kind),
                ExprKind::Call { .. } | ExprKind::Identifier(_) => false,
            }
        }

        let is_const = is_const(&kind);

        Self {
            id,
            line,
            kind,
            symtable,

            typ: None,
            is_const,
        }
    }

    /// Useful for testing
    pub fn without_table(id: i32, kind: ExprKind) -> Self {
        Self::new(id, 0, kind, SymbolTable::new())
    }

    pub fn as_node(&self) -> AstNode {
        AstNode::Expr(self)
    }

    pub fn children(&self) -> impl Iterator<Item = AstNode> {
        let mut children = Vec::new();

        match &self.kind {
            ExprKind::Grouping(inner) => children.push(inner.as_node()),
            ExprKind::BinaryOp { lhs, rhs, .. } => {
                children.push(lhs.as_node());
                children.push(rhs.as_node());
            }
            ExprKind::UnaryOp { value, .. } => children.push(value.as_node()),
            ExprKind::Call { callee, args } => {
                children.push(callee.as_node());
                children.extend(args.iter().map(Expr::as_node));
            }
            _ => (),
        }

        children.into_iter()
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

#[derive(Clone, Debug)]
pub struct Stmt {
    pub id: i32,
    pub line: u32,
    pub kind: StmtKind,
    pub symtable: SymbolTable,
}

impl PartialEq for Stmt {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.kind == other.kind
    }
}

impl Stmt {
    pub fn new(id: i32, line: u32, kind: StmtKind, symtable: SymbolTable) -> Self {
        Self {
            id,
            line,
            kind,
            symtable,
        }
    }

    /// Useful for testing
    pub fn without_table(id: i32, kind: StmtKind) -> Self {
        Self {
            id,
            line: 0,
            kind,
            symtable: SymbolTable::new(),
        }
    }

    pub fn as_node(&self) -> AstNode {
        AstNode::Stmt(self)
    }

    pub fn children(&self) -> impl Iterator<Item = AstNode> {
        let mut children = Vec::new();

        match &self.kind {
            StmtKind::Block(inner) => {
                children.extend(inner.iter().map(Self::as_node));
            }
            StmtKind::ExprStmt(expr) => children.push(expr.as_node()),
            StmtKind::IfStmt {
                condition,
                if_then,
                else_then,
            } => {
                children.push(condition.as_node());
                children.push(if_then.as_node());
                if let Some(else_then) = else_then {
                    children.push(else_then.as_node());
                }
            }
            StmtKind::WhileStmt { condition, body } => {
                children.push(condition.as_node());
                children.push(body.as_node());
            }
            StmtKind::ForStmt {
                iterator,
                identifier: _,
                body,
            } => {
                children.push(iterator.as_node());
                children.push(body.as_node());
            }
            StmtKind::DefineVariable { value, .. } => children.push(value.as_node()),
            StmtKind::DefineValue { value, .. } => children.push(value.as_node()),
            StmtKind::AssignVariable { value, .. } => children.push(value.as_node()),
            StmtKind::DefineFunction(Function { kind, .. }) => {
                if let FunctionKind::Normal { body } = kind {
                    children.push(body.as_node())
                }
            }
            StmtKind::Return(value) => children.push(value.as_node()),
        }

        children.into_iter()
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
    ForStmt {
        iterator: Expr,
        identifier: String,
        body: Box<Stmt>,
    },
    DefineVariable {
        identifier: String,
        value: Expr,
        typ: Option<TypeIdentifier>,
    },
    DefineValue {
        identifier: String,
        value: Expr,
        typ: Option<TypeIdentifier>,
    },
    AssignVariable {
        identifier: String,
        value: Expr,
    },
    /// A function definition. Output is None when the function returns nothing
    /// meaning void, otherwise it is the name of the type the function
    /// returns.
    DefineFunction(Function),
    Return(Expr),
}

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub identifier: String,
    pub inputs: Vec<FunctionInput>,
    pub output: Option<TypeIdentifier>,
    pub kind: FunctionKind,
}

#[derive(PartialEq, Clone, Debug)]
pub enum FunctionKind {
    Normal { body: Box<Stmt> },
    Foreign,
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionInput {
    pub identifier: String,
    pub typ: TypeIdentifier,
}

#[derive(PartialEq, Clone, Debug)]
pub struct TypeIdentifier {
    pub name: String,
    pub is_list: bool,
}

impl Display for TypeIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_list {
            write!(f, "[")?;
        }

        write!(f, "{}", self.name)?;

        if self.is_list {
            write!(f, "]")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Character(char),
    String(String),
    Array(Vec<Expr>),
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
            Literal::String(s) => format!(
                "\\\"{}\\\"",
                s.replace('\"', "\\\"").replace("\\n", "\\\\n")
            ),
            Literal::Array(..) => "<Array>".to_string(),
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

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            BinaryOp::Add => "+",
            BinaryOp::Con => "++",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",

            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::EqEq => "==",
            BinaryOp::NotEq => "!=",

            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",

            BinaryOp::Range => "..",
        };

        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,

    Reference,
    Dereference,
}

impl TryFrom<TokenType> for UnaryOp {
    type Error = ParsingError;

    fn try_from(value: TokenType) -> Result<Self, Self::Error> {
        let operation = match value {
            TokenType::Bang => Self::Not,
            TokenType::Minus => Self::Neg,

            TokenType::Star => Self::Reference,
            TokenType::At => Self::Dereference,

            _ => return Err(ParsingError::InvalidOp),
        };

        Ok(operation)
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",

            UnaryOp::Reference => "*",
            UnaryOp::Dereference => "@",
        };

        write!(f, "{value}")
    }
}
