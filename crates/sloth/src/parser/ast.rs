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
    Range,
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
#[derive(PartialEq, Debug)]
pub struct FuncArgs {
    pub name: String,
    pub typ: Option<String>,
}
#[derive(PartialEq, Debug)]
pub enum Stmt {
    ExprStmt(Expr),
    DefineFunction {
        ident: String,
        args: Option<Vec<FuncArgs>>,
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
    AssignVariable {
        name: String,
        value: Expr,
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
    Return {
        value: Expr,
    },
}
