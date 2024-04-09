use ast::Module;
use bayou_session::Interner;
use lexer::Lexer;

#[macro_use]
extern crate macro_rules_attribute;

mod lexer;
mod parser;

pub mod ast;
pub mod token;

pub use lexer::{LexerError, LexerErrorKind, LexerResult, TokenIter};
pub use parser::ParseError;
use parser::Parser;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

pub fn lex(source: &str, interner: &mut Interner) -> (TokenIter, Vec<LexerError>) {
    let lexer = Lexer::new(source, interner);
    lexer.lex()
}

pub fn parse(tokens: TokenIter) -> (Module, Vec<ParseError>) {
    let parser = Parser::new(tokens);
    parser.parse()
}
