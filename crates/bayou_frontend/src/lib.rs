#[macro_use]
extern crate macro_rules_attribute;

mod lexer;
mod parser;

mod module_global_lookup;
mod module_loader;

mod lower;

pub mod ast;
pub mod token;

pub use lexer::{LexerError, LexerErrorKind, LexerResult, TokenIter};
pub use lower::NameError;
pub use parser::ParseError;

use ast::Module;
use bayou_interner::Interner;
use bayou_session::sourcemap::SourceId;
use lexer::Lexer;
use parser::Parser;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

pub fn lex(source: &str, interner: &Interner) -> (TokenIter, Vec<LexerError>) {
    Lexer::new(source, interner).lex()
}

pub fn parse(tokens: TokenIter) -> (Module, Vec<ParseError>) {
    Parser::new(tokens).parse()
}

pub fn lower(
    ast: ast::Module,
    symbols: &mut bayou_ir::symbols::Symbols,
    source_id: SourceId,
) -> Result<bayou_ir::ir::PackageIr, Vec<NameError>> {
    lower::Lowerer::new(symbols, source_id).run(ast)
}

pub enum LoadAndParseError {
    LexerError(LexerError),
    ParseError(ParseError),
}

// pub fn load_and_parse_modules(interner: &mut Interner)
