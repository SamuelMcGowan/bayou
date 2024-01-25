#[cfg(test)]
mod tests;

mod lexer;
mod parser;
mod resolver;

use self::parser::Parser;
use self::resolver::Resolver;
use crate::ir::ast::Module;
use crate::{CompilerError, CompilerResult};

pub fn run_frontend(source: &str) -> CompilerResult<Module> {
    let mut parser = Parser::new(source);
    let mut ast = parser.parse_module();

    let (interner, parser_diagnostics) = parser.finish();

    if !parser_diagnostics.is_empty() {
        return Err(CompilerError::HadErrors);
    }

    let resolver = Resolver::new(&interner);
    let resolver_diagnostics = resolver.run(&mut ast);

    if !resolver_diagnostics.is_empty() {
        return Err(CompilerError::HadErrors);
    }

    Ok(ast)
}
