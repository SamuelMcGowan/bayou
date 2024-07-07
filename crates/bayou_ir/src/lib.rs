//! Types for representing the program throughout the compilation pipeline.

#[macro_use]
extern crate macro_rules_attribute;

pub mod ir;
pub mod symbols;

use bayou_interner::Istr;
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

#[derive(NodeCopyTraits!)]
pub struct Ident {
    pub istr: Istr,
    pub span: Span,
}

impl Ident {
    pub fn with_source(self, source_id: SourceId) -> IdentWithSource {
        IdentWithSource {
            istr: self.istr,
            span: SourceSpan::new(self.span, source_id),
        }
    }
}

#[derive(NodeCopyTraits!)]
pub struct IdentWithSource {
    pub istr: Istr,
    pub span: SourceSpan,
}
