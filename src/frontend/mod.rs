#[cfg(test)]
mod tests;

// FIXME: hide internals
mod ast;
mod lexer;
mod lower;
mod parser;

use self::lower::Lowerer;
use self::parser::Parser;
use crate::ir::ssa::ModuleIr;
use crate::session::{Session, Symbols};
use crate::{CompilerError, CompilerResult};

pub fn parse_and_build_ir(session: &Session, source: &str) -> CompilerResult<(ModuleIr, Symbols)> {
    let parser = Parser::new(session, source);
    let ast = parser.parse_module();

    if session.had_errors() {
        return Err(CompilerError::HadErrors);
    }

    let lowerer = Lowerer::default();
    let (ir, symbols) = lowerer.run(ast);

    Ok((ir, symbols))
}
