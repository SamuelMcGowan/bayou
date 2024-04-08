use bayou_diagnostic::span::Span;
use bayou_session::sourcemap::SourceId;

use super::{BinOp, Ident, Node, NodeCopy, UnOp};
use crate::symbols::{LocalId, Symbols};
use crate::Type;

#[derive(Node!)]
pub struct Module {
    pub items: Vec<Item>,
}

// TODO: revisit borrowing issues
/// Additional information about a module that can't be stored in the [`Module`] type due
/// to borrowing issues.
pub struct ModuleContext {
    pub source_id: SourceId,
    pub symbols: Symbols,
}

#[derive(Node!)]
pub enum Item {
    FuncDecl(FuncDecl),
}

#[derive(Node!)]
pub struct FuncDecl {
    pub name: Ident,
    pub ret_ty: Type,
    pub statements: Vec<Stmt>,
}

#[derive(Node!)]
pub enum Stmt {
    Assign { local: LocalId, expr: Expr },
    Return(Expr),
}

#[derive(Node!)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<Type>,
}

#[derive(Node!)]
pub enum ExprKind {
    Constant(Constant),
    Var(LocalId),
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Void,
}

#[derive(NodeCopy!)]
pub enum Constant {
    I64(i64),
}

impl Constant {
    pub fn ty(&self) -> Type {
        match self {
            Self::I64(_) => Type::I64,
        }
    }
}
