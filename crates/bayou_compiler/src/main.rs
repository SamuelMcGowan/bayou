#[macro_use]
extern crate macro_rules_attribute;

mod parser;

mod cli;
mod compiler;
mod diagnostics;
mod ir;
mod sourcemap;
mod symbols;
mod utils;

use clap::Parser as _;
use cli::{Cli, Command};

use crate::compiler::Compiler;
use crate::diagnostics::PrettyDiagnosticEmitter;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

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
        Command::Build { input, output } => {
            println!("building file {}...\n", input.display());

            let source = std::fs::read_to_string(&input)?;

            let mut compiler = Compiler::new(PrettyDiagnosticEmitter::default());
            compiler.parse_module(input.to_string_lossy(), source)?;

            Ok(())
        }
    }
}
