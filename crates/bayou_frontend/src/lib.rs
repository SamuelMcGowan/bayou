#[macro_use]
extern crate macro_rules_attribute;

mod lexer;
mod parser;

mod gather_modules;
mod module_tree;

mod lower;

pub mod ast;
pub mod token;

use gather_modules::{GatherModulesError, ModuleGatherer, ParsedModule};
pub use lexer::{LexerError, LexerErrorKind, LexerResult, TokenIter};
pub use lower::NameError;
pub use parser::ParseError;

use ast::Module;
use bayou_interner::Interner;
use bayou_session::{module_loader::ModuleLoader, sourcemap::SourceMap};
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

pub fn load_and_parse_modules<M: ModuleLoader>(
    source_map: &mut SourceMap,

    module_loader: &M,
    interner: &Interner,
) -> (ModuleTree, Vec<ParsedModule>, Vec<GatherModulesError<M>>) {
    ModuleGatherer::new(source_map, module_loader, interner).run()
}

pub fn lower(
    modules: &[ParsedModule],
    module_tree: &ModuleTree,
    symbols: &mut bayou_ir::symbols::Symbols,
) -> Result<bayou_ir::ir::PackageIr, Vec<NameError>> {
    let mut errors = vec![];
    let mut package_ir = bayou_ir::ir::PackageIr::default();

    for module in modules {
        lower::ModuleLowerer::new(module, module_tree, symbols, &mut package_ir, &mut errors).run();
    }

    if errors.is_empty() {
        Ok(package_ir)
    } else {
        Err(errors)
    }
}
