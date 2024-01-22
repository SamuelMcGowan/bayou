#[cfg(test)]
mod tests;

// FIXME: hide internals
mod ast;
mod lexer;
mod lower;
mod parser;

use self::lower::lower;
use self::parser::Parser;
use crate::ir::ssa::ModuleIr;
use crate::session::Session;
use crate::{CompilerError, CompilerResult};

pub fn parse_and_build_ir(session: &Session, source: &str) -> CompilerResult<ModuleIr> {
    let parser = Parser::new(session, source);
    let ast = parser.parse_module();

    if session.had_errors() {
        return Err(CompilerError::HadErrors);
    }

    Ok(lower(ast))
}
