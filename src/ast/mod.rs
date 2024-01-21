pub type InternedStr = lasso::Spur;
pub type Interner = lasso::Rodeo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(InternedStr),
    Integer(u64),

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Int,
    Return,
}
