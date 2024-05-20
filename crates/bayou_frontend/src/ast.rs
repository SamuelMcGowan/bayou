use bayou_diagnostic::span::Span;
use bayou_ir::{BinOp, Type, UnOp};
use bayou_session::Ident;

use crate::Node;

#[derive(Node!)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Node!)]
pub enum Item {
    FuncDecl(FuncDecl),
    ParseError,
}

#[derive(Node!)]
pub struct FuncDecl {
    pub ident: Ident,
    pub ret_ty: Type,
    pub statements: Vec<Stmt>,
}

#[derive(Node!)]
pub enum Stmt {
    Assign { ident: Ident, ty: Type, expr: Expr },
    Drop(Expr),
    Return(Expr),

    ParseError,
}

#[derive(Node!)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Node!)]
pub enum ExprKind {
    Integer(i64),
    Bool(bool),

    Var(Ident),

    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },

    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },

    Void,

    ParseError,
}

impl ExprKind {
    pub fn requires_semicolon_if_stmt(&self) -> bool {
        match self {
            ExprKind::If { .. } => false,
            _ => false,
        }
    }
}
