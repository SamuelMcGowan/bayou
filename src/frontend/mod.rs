#[cfg(test)]
mod tests;

mod lexer;
mod parser;

use self::parser::Parser;
use crate::ast::Module;
use crate::session::Session;
use crate::{CompilerError, CompilerResult};

pub fn run_frontend(session: &Session, source: &str, print_output: bool) -> CompilerResult<Module> {
    let parser = Parser::new(session, source);
    let ast = parser.parse_module();

    if session.diagnostics.had_errors() {
        if print_output {
            session.diagnostics.flush_diagnostics();
        }

        return Err(CompilerError::HadErrors);
    }

    Ok(ast)
}
