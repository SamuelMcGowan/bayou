#[macro_use]
extern crate macro_rules_attribute;

mod frontend;

mod cli;
mod compiler;
mod diagnostics;
mod ir;
mod symbols;
mod utils;

use bayou_diagnostic::termcolor::{ColorChoice, StandardStream};
use bayou_diagnostic::Config;
use clap::Parser as _;
use cli::{Cli, Command};

use crate::compiler::Compiler;

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

            let mut compiler = Compiler::default();
            let mut diagnostics = compiler.parse_module(input.to_string_lossy(), source);

            // if diagnostics.flush(
            //     &compiler.sources,
            //     &Config::default(),
            //     &mut StandardStream::stderr(ColorChoice::Auto),
            // )? {
            //     return Err(CompilerError::HadErrors);
            // }

            if !diagnostics.is_empty() {
                return Err(CompilerError::HadErrors);
            }

            // if let Some(path) = output {
            //     println!("writing assembly to {}", path.display());
            //     std::fs::write(path, asm)?;
            // } else {
            //     println!("ASSEMBLY OUTPUT:\n\n{asm}");
            // }

            Ok(())
        }
    }
}
