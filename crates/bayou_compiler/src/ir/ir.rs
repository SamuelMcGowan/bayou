use super::ast::Ident;
pub use super::ast::{BinOp, UnOp}; // TODO: move ops to their own module.
use super::Node;
use crate::symbols::LocalId;

pub struct FuncDecl {
    pub name: Ident,
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
    pub ty: Type,
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
}

#[derive(Node!, Copy, Hash)]
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

#[derive(Node!, Copy, Hash)]
pub enum Type {
    I64,
}
