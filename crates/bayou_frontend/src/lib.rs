#[macro_use]
extern crate macro_rules_attribute;

pub mod lexer;
pub mod parser;

pub mod ast;
pub mod token;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}
