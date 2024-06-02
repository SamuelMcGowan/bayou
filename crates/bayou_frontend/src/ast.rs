use bayou_diagnostic::span::Span;
use bayou_ir::{BinOp, Spanned, Type, UnOp};
use bayou_session::InternedStr;

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
    pub ident: Spanned<InternedStr>,
    pub ret_ty: Type,
    pub block: Block,
}

#[derive(Node!)]
pub enum Stmt {
    Assign {
        ident: Spanned<InternedStr>,
        ty: Type,
        expr: Expr,
    },
    Drop {
        expr: Expr,
        had_semicolon: bool,
    },
    Return(Expr),

    ParseError,
}

#[derive(Node!)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub final_expr: Expr,
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

    Var(Spanned<InternedStr>),

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
