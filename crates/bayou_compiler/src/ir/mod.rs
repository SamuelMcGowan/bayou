use bayou_diagnostic::span::Span;

pub mod ast;
#[allow(clippy::module_inception)]
pub mod ir;
pub mod token;

pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

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
