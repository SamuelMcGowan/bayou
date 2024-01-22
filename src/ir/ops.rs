#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    CmpOp(CmpOp),

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CmpOp {
    Eq,
    NotEq,

    Gt,
    Lt,
    GtEq,
    LtEq,
}
