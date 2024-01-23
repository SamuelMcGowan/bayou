#[cfg(test)]
mod tests;

mod lexer;
mod parser;
mod resolver;

use self::parser::Parser;
use self::resolver::Resolver;
use crate::ir::ast::Module;
use crate::session::Session;
use crate::{CompilerError, CompilerResult};

pub fn run_frontend(session: &Session, source: &str) -> CompilerResult<Module> {
    let parser = Parser::new(session, source);
    let mut ast = parser.parse_module();

    if session.diagnostics.had_errors() {
        return Err(CompilerError::HadErrors);
    }

    let resolver = Resolver::new(session);
    resolver.run(&mut ast);

    if session.diagnostics.had_errors() {
        return Err(CompilerError::HadErrors);
    }

    Ok(ast)
}
