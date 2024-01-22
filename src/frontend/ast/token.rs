use crate::session::InternedStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(InternedStr),
    Integer(u64),

    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Int,
    Return,
}
