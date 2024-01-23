use super::vars::PlaceRef;
use super::{Node, NodeCopy};
use crate::session::InternedStr;

#[derive(Node!)]
pub struct Module {
    pub item: Item,
}

#[derive(Node!)]
pub enum Item {
    FuncDecl(FuncDecl),
    ParseError,
}

#[derive(Node!)]
pub struct FuncDecl {
    pub name: InternedStr,
    pub statement: Stmt,
}

#[derive(Node!)]
pub enum Stmt {
    Return(Expr),
    ParseError,
}

#[derive(Node!)]
pub struct Expr {
    pub kind: ExprKind,
    pub place: Option<PlaceRef>,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self { kind, place: None }
    }
}

#[derive(Node!)]
pub enum ExprKind {
    Constant(u64),

    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    ParseError,
}

#[derive(NodeCopy!)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    // Eq,
    // NotEq,

    // Gt,
    // Lt,
    // GtEq,
    // LtEq,
}

#[derive(NodeCopy!)]
pub enum UnOp {
    Negate,
    BitwiseInvert,
}
