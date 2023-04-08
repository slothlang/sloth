pub enum BinaryOp {
    Add,
    Con,
    Sub,
    Mul,
    Pow,
    Div,

    BWSftRight,
    BWSftLeft,
    BWAnd,
    BWOr,
    BWXor,
}

pub enum UnaryOp {
    Not,
    Neg,

    BWComp,
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
}
