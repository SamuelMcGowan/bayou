use clap::Parser as _;
use cli::{Cli, Command};
use session::Session;

use crate::backend::run_backend;
use crate::frontend::run_frontend;

mod backend;
mod frontend;
mod ir;

mod cli;
mod session;
mod utils;

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
            println!("building file {}", input.display());
            let source = std::fs::read_to_string(input)?;
            let asm = compile(&source, true)?;

            if let Some(path) = output {
                println!("writing assembly to {}", path.display());
                std::fs::write(path, asm)?;
            } else {
                println!("ASSEMBLY OUTPUT:\n\n{asm}");
            }

            Ok(())
        }

        #[cfg(feature = "test_suite")]
        Command::RunTestSuite { stage } => test_suite::run_test_suite(stage),
    }
}

fn compile(source: &str, print_output: bool) -> CompilerResult<String> {
    let session = Session::default();

    let (ir, symbols) = run_frontend(&session, source, print_output)?;

    let asm = run_backend(&session, symbols, &ir, print_output);

    Ok(asm)
}
