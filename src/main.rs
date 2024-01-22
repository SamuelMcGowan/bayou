use clap::Parser as _;
use cli::{Cli, Command};
use session::Session;

use crate::frontend::run_frontend;

mod frontend;

mod ast;
mod cli;
mod session;
pub mod symbols;
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
    }
}

fn compile(source: &str, print_output: bool) -> CompilerResult<String> {
    let session = Session::default();

    let ast = run_frontend(&session, source)?;

    if session.diagnostics.had_errors() {
        if print_output {
            session.diagnostics.flush_diagnostics();
        }

        return Err(CompilerError::HadErrors);
    }

    if print_output {
        println!("{session:#?}");
    }

    Ok(format!("{ast:#?}"))
}
