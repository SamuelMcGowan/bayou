use bayou_diagnostic::span::Span;

use crate::ir::InternedStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedStr),
    Integer(i64),

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

    Assign,
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
    Let,
}

impl TokenKind {
    pub fn token_name(&self) -> &'static str {
        match self {
            TokenKind::Keyword(kw) => match kw {
                Keyword::Int => "keyword `int`",
                Keyword::Return => "keyword `return`",
                Keyword::Let => "keyword `let`",
            },
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Integer(_) => "integer",
            TokenKind::LBrace => "`{`",
            TokenKind::RBrace => "`}`",
            TokenKind::LParen => "`(`",
            TokenKind::RParen => "`)`",
            TokenKind::Semicolon => "`;`",
            TokenKind::Add => "`+`",
            TokenKind::Sub => "`-`",
            TokenKind::Mul => "`*`",
            TokenKind::Div => "`/`",
            TokenKind::Mod => "`%`",
            TokenKind::Assign => "`=`",
            TokenKind::BitwiseAnd => "`&`",
            TokenKind::BitwiseOr => "`|`",
            TokenKind::BitwiseXor => "`^`",
            TokenKind::BitwiseInvert => "`~`",
        }
    }
}
