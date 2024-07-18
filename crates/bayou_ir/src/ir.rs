use bayou_interner::Interner;
use bayou_session::sourcemap::SourceSpan;

use super::{BinOp, NodeCopyTraits, NodeTraits, UnOp};
use crate::symbols::{FuncId, LocalId, Symbols};
use crate::Type;

pub struct Package {
    pub name: String,

    pub ir: PackageIr,
    pub symbols: Symbols,
    pub interner: Interner,
}

#[derive(Default, NodeTraits!)]
pub struct PackageIr {
    pub items: Vec<Item>,
    pub main_func: Option<FuncId>,
}

#[derive(NodeTraits!)]
pub enum Item {
    FuncDecl(FuncDecl),
}

#[derive(NodeTraits!)]
pub struct FuncDecl {
    pub id: FuncId,
    pub block: Block,
}

#[derive(NodeTraits!)]
pub enum Stmt {
    Assign { local: LocalId, expr: Expr },
    Drop(Expr),
    Return(Expr),
}

#[derive(NodeTraits!)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub final_expr: Expr,

    pub span: SourceSpan,
}

#[derive(NodeTraits!)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
    pub ty: Option<Type>,
}

#[derive(NodeTraits!)]
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
    Block(Box<Block>),
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },
}

#[derive(NodeCopyTraits!)]
pub enum Constant {
    I64(i64),
    Bool(bool),
    Void,
}

impl Constant {
    pub fn ty(&self) -> Type {
        match self {
            Self::I64(_) => Type::I64,
            Self::Bool(_) => Type::Bool,
            Self::Void => Type::Void,
        }
    }
}
