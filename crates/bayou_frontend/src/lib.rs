use ast::Module;
use bayou_interner::Interner;
use lexer::Lexer;

#[macro_use]
extern crate macro_rules_attribute;

mod lexer;
mod lower;
mod parser;

pub mod ast;
pub mod token;

pub use lexer::{LexerError, LexerErrorKind, LexerResult, TokenIter};
pub use lower::NameError;
pub use parser::ParseError;
use parser::Parser;

derive_alias! {
    #[derive(Node!)] = #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)];
    #[derive(NodeCopy!)] = #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)];
}

pub fn lex(source: &str, interner: &mut Interner) -> (TokenIter, Vec<LexerError>) {
    Lexer::new(source, interner).lex()
}

pub fn parse(tokens: TokenIter) -> (Module, Vec<ParseError>) {
    Parser::new(tokens).parse()
}

pub fn lower(
    ast: ast::Module,
) -> Result<(bayou_ir::ir::Module, bayou_ir::symbols::Symbols), Vec<NameError>> {
    let mut symbols = bayou_ir::symbols::Symbols::default();
    lower::Lowerer::new(&mut symbols)
        .run(ast)
        .map(|ir| (ir, symbols))
}

pub fn lower_new(
    ast: ast::Module,
    symbols: &mut bayou_ir::symbols::Symbols,
) -> Result<bayou_ir::ir::Module, Vec<NameError>> {
    lower::Lowerer::new(symbols).run(ast)
}
