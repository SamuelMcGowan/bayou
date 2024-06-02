//! Types for representing the program throughout the compilation pipeline.

#[macro_use]
extern crate macro_rules_attribute;

pub mod ir;
pub mod symbols;

use bayou_diagnostic::span::Span;
use bayou_session::InternedStr;

derive_alias! {
    #[derive(NodeTraits!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopyTraits!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

#[derive(NodeCopyTraits!)]
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

#[derive(NodeCopyTraits!)]
pub enum UnOp {
    Negate,
    BitwiseInvert,
}

#[derive(NodeCopyTraits!)]
pub enum Type {
    I64,
    Bool,
    Void,
    Never,
}

#[derive(NodeCopyTraits!)]
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
