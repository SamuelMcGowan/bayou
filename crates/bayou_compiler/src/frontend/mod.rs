#[cfg(test)]
mod tests;

mod lexer;
mod parser;
mod resolver;

use bayou_diagnostic::sources::Cached;
use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::Config;

use self::parser::Parser;
use self::resolver::Resolver;
use crate::ir::ast::Module;
use crate::{CompilerError, CompilerResult};

pub fn run_frontend(source: &str) -> CompilerResult<Module> {
    let diagnostics_config = Config::default();
    let mut diagnostics_stream = StandardStream::stderr(ColorChoice::Auto);

    let sources = vec![Cached::new(("a module".to_owned(), source.to_owned()))];

    let mut parser = Parser::new(source);
    let mut ast = parser.parse_module();

    let (interner, parser_diagnostics) = parser.finish();

    parser_diagnostics.emit(&sources, &diagnostics_config, &mut diagnostics_stream)?;

    let resolver = Resolver::new(&interner);
    let resolver_diagnostics = resolver.run(&mut ast);

    if resolver_diagnostics.had_errors() {
        return Err(CompilerError::HadErrors);
    }

    Ok(ast)
}
