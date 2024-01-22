mod codegen;
pub mod regalloc;
pub mod registers;

use self::codegen::CodeGenerator;
use crate::ir::ssa::ModuleIr;
use crate::session::{Session, Symbols};

pub fn run_backend(
    session: &Session,
    symbols: Symbols,
    module: &ModuleIr,
    print_output: bool,
) -> String {
    let symbols = regalloc::regalloc(symbols, module);

    if print_output {
        println!("ir: {module:?}");
        println!("symbols: {symbols:?}");
    }

    let code_generator = CodeGenerator::new(session, symbols);
    code_generator.run(module)
}
