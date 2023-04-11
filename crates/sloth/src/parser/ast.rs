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

pub enum UnaryOp {
    Not,
    Neg,

    BWComp,
}

pub enum Literal {
    Integer(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Regex(String), 
    List(Vec<Expr>), // TODO: holy shit we forgor empty listys
}

pub enum Expr {
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
        expr: Vec<Expr>,
        body: Vec<Stmt>,
        else_if: Vec<(Expr, Stmt)>,
        els: Option<Box<Stmt>>,
    },
    For {
        name: Expr,
        iter: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}