use crate::session::InternedStr;

pub mod token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub item: Item,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    FuncDecl(FuncDecl),
    ParseError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncDecl {
    pub name: InternedStr,
    pub statement: Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Return(Expression),
    ParseError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Constant(u64),
}
