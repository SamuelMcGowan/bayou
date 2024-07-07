//! Types for representing the program throughout the compilation pipeline.

#[macro_use]
extern crate macro_rules_attribute;

pub mod ir;
pub mod symbols;

use bayou_session::{
    diagnostics::span::Span,
    sourcemap::{SourceId, SourceSpan},
};

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

// pub type Spanned<T> = WithSpan<T, Span>;
// pub type SourceSpanned<T> = Spanned<T, SourceSpan>;

#[derive(NodeCopyTraits!)]
pub struct Spanned<T, S = Span> {
    pub node: T,
    pub span: S,
}

impl<T> Spanned<T, Span> {
    pub fn to_source_spanned(self, source_id: SourceId) -> Spanned<T, SourceSpan> {
        Spanned {
            node: self.node,
            span: SourceSpan::new(self.span, source_id),
        }
    }
}

impl<T, S> Spanned<T, S> {
    pub fn new(node: T, span: S) -> Self {
        Self { node, span }
    }
}

impl<T, E, Extra> Spanned<Result<T, E>, Extra> {
    pub fn transpose(self) -> Result<Spanned<T, Extra>, E> {
        self.node.map(|node| Spanned {
            node,
            span: self.span,
        })
    }
}
