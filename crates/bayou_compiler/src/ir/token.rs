use bayou_diagnostic::span::Span;

use super::NodeCopy;
use crate::ir::InternedStr;

#[derive(NodeCopy!)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(NodeCopy!)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(InternedStr),
    Integer(i64),

    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
    Bang,
    Arrow,

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

#[derive(NodeCopy!)]
pub enum Keyword {
    Func,
    Return,

    Let,

    Void,
    I64,
}

impl TokenKind {
    pub fn token_name(&self) -> &'static str {
        match self {
            TokenKind::Keyword(kw) => match kw {
                Keyword::Func => "keyword `func`",
                Keyword::Return => "keyword `return`",
                Keyword::Let => "keyword `let`",
                Keyword::I64 => "keyword `i64`",
                Keyword::Void => "keyword `void`",
            },
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Integer(_) => "integer",
            TokenKind::LBrace => "`{`",
            TokenKind::RBrace => "`}`",
            TokenKind::LParen => "`(`",
            TokenKind::RParen => "`)`",
            TokenKind::Semicolon => "`;`",
            TokenKind::Bang => "`!`",
            TokenKind::Arrow => "`->`",
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
