#[macro_use]
extern crate macro_rules_attribute;

mod lexer;
mod parser;

mod gather_modules;
mod module_tree;

mod lower;

pub mod ast;
pub mod token;

use bayou_ir::symbols::Symbols;
pub use gather_modules::GatherModulesError;
use gather_modules::{ModuleGatherer, ParsedModule};
pub use lexer::{LexerError, LexerErrorKind, LexerResult, TokenIter};
pub use lower::NameError;
pub use parser::ParseError;

use ast::Module;
use bayou_interner::Interner;
use bayou_session::{PackageSession, Session};
use lexer::Lexer;
use module_tree::ModuleTree;
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

pub fn load_and_parse_modules<S: Session>(
    session: &mut S,
    package_session: &mut PackageSession<S>,
) -> (ModuleTree, Vec<ParsedModule>, Vec<GatherModulesError>) {
    ModuleGatherer::new(session, package_session).run()
}

pub fn lower(
    modules: &[ParsedModule],
    module_tree: &mut ModuleTree,
    interner: &Interner,
) -> (
    bayou_ir::ir::PackageIr,
    bayou_ir::symbols::Symbols,
    Vec<NameError>,
) {
    let mut errors = vec![];

    let mut symbols = Symbols::default();
    let mut package_ir = bayou_ir::ir::PackageIr::default();

    for module in modules {
        lower::ModuleLowerer::new(
            module,
            module_tree,
            &mut symbols,
            &mut package_ir,
            &mut errors,
            interner,
        )
        .run();
    }

    (package_ir, symbols, errors)
}
