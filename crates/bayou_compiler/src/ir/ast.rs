use super::{BinOp, Ident, Node, UnOp};
use crate::symbols::LocalId;

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
    pub name: Ident,
    pub statements: Vec<Stmt>,
}

#[derive(Node!)]
pub enum Stmt {
    Assign { ident: Ident, expr: Expr },

    Return(Expr),

    ParseError,
}

#[derive(Node!)]
pub enum Expr {
    Constant(i64),

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

    ParseError,
}
