use bayou_ir::{BinOp, Ident, Type, UnOp};
use bayou_session::diagnostics::span::Span;

use crate::Node;

#[derive(Node!, Default)]
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
    pub ret_ty_span: Span,

    pub block: Block,
}

#[derive(Node!)]
pub enum Stmt {
    Assign { ident: Ident, ty: Type, expr: Expr },
    Drop { expr: Expr, had_semicolon: bool },
    Return(Expr),

    ParseError,
}

#[derive(Node!)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub final_expr: Expr,

    pub span: Span,
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

    Block(Box<Block>),

    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },

    Void,

    ParseError,
}

impl ExprKind {
    /// Whether a semicolon is optional after an expression statement
    /// of this kind.
    pub fn stmt_semicolon_is_optional(&self) -> bool {
        matches!(self, ExprKind::Block(_) | ExprKind::If { .. })
    }
}
