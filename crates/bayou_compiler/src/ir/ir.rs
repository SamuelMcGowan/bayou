use super::{BinOp, Ident, Node, NodeCopy, UnOp};
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

#[derive(NodeCopy!)]
pub enum Type {
    I64,
}
