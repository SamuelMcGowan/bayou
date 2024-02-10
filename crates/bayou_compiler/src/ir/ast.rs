use bayou_diagnostic::span::Span;

use super::vars::PlaceRef;
use super::{Node, NodeCopy};
use crate::ir::InternedStr;

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
    pub name: Ident,
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
    Constant(i64),

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

#[derive(Node!, Copy)]
pub struct Ident {
    pub ident: InternedStr,
    pub span: Span,
}
