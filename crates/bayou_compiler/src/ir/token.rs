use crate::diagnostics::span::Span;
use crate::session::InternedStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedStr),
    Integer(u64),

    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // TODO: uncomment when assignment is added
    // AddEq,
    // SubEq,
    // MulEq,
    // DivEq,
    // ModEq,

    // TODO: uncomment when boolean operations are added
    // EqEq,
    // NotEq,

    // Gt,
    // Lt,
    // GtEq,
    // LtEq,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseInvert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Int,
    Return,
}
