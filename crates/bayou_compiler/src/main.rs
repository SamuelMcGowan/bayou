#[macro_use]
extern crate macro_rules_attribute;

mod parser;

mod cli;
mod codegen;
mod compiler;
mod diagnostics;
mod ir;
mod sourcemap;
mod symbols;
mod utils;

use clap::Parser as _;
use cli::{Cli, Command};
use codegen::CodegenError;
use target_lexicon::Triple;

use crate::compiler::Compiler;
use crate::diagnostics::PrettyDiagnosticEmitter;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("error during codegen: {0}")]
    Codegen(#[from] CodegenError),

    #[error("error emitting object: {0}")]
    Object(#[from] cranelift_object::object::write::Error),

    #[error("errors while compiling")]
    HadErrors,
}

type CompilerResult<T> = Result<T, CompilerError>;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}

fn run() -> CompilerResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build {
            input,
            source,
            output,
        } => {
            let (name, source) = if source {
                ("<unnamed>".to_owned(), input)
            } else {
                let source = std::fs::read_to_string(&input)?;
                (input, source)
            };

            println!("building file {name}...\n");

            let triple = Triple::host();
            let mut compiler = Compiler::new(PrettyDiagnosticEmitter::default(), triple);

            // TODO: store modules in module tree. Maybe two trees, one for ast and
            // one for contexts? Would allow borrowing other module contexts
            // while traversing asts.
            let (module, module_cx) = compiler.parse_module(&name, source)?;
            let object = compiler.compile(&name, &module, &module_cx)?;

            let output_path = output.unwrap_or_else(|| {
                // TODO: make better filename
                name.to_owned() + ".o"
            });

            let object_data = object.emit()?;
            std::fs::write(output_path, object_data)?;

            Ok(())
        }
    }
}
