use bayou_diagnostic::span::Span;
use bayou_interner::Istr;

use crate::NodeCopy;

#[derive(NodeCopy!)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(NodeCopy!)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(Istr),
    Integer(i64),
    Bool(bool),

    LBrace,
    RBrace,
    LParen,
    RParen,

    Dot,
    Colon,
    Comma,
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
    If,
    Then,
    Else,

    I64,
    Bool,
    Void,
}

impl TokenKind {
    pub fn token_name(&self) -> &'static str {
        match self {
            TokenKind::Keyword(kw) => match kw {
                Keyword::Func => "keyword `func`",
                Keyword::Return => "keyword `return`",
                Keyword::Let => "keyword `let`",
                Keyword::If => "keyword `if`",
                Keyword::Then => "keyword `then`",
                Keyword::Else => "keyword `else`",
                Keyword::I64 => "keyword `i64`",
                Keyword::Bool => "keyword `bool`",
                Keyword::Void => "keyword `void`",
            },
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Integer(_) => "integer",
            TokenKind::Bool(_) => "boolean",
            TokenKind::LBrace => "`{`",
            TokenKind::RBrace => "`}`",
            TokenKind::LParen => "`(`",
            TokenKind::RParen => "`)`",
            TokenKind::Dot => "`.`",
            TokenKind::Colon => "`:`",
            TokenKind::Comma => "`,`",
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
