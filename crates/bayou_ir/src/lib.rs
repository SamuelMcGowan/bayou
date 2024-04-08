#[macro_use]
extern crate macro_rules_attribute;

pub mod ast;
pub mod ir;
pub mod symbols;
pub mod token;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

use bayou_diagnostic::span::Span;
pub use lasso;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Node!, Copy)]
pub struct Ident {
    pub ident: InternedStr,
    pub span: Span,
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

#[derive(NodeCopy!)]
pub enum Type {
    I64,
    Void,
    Never,
}

#[derive(NodeCopy!)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

impl<T, E> Spanned<Result<T, E>> {
    pub fn transpose(self) -> Result<Spanned<T>, E> {
        self.node.map(|node| Spanned {
            node,
            span: self.span,
        })
    }
}
