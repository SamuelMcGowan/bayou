#[macro_use]
extern crate macro_rules_attribute;

mod frontend;

mod cli;
mod diagnostic;
mod ir;
mod session;
mod symbols;
mod utils;

use clap::Parser as _;
use cli::{Cli, Command};
use session::Session;

use crate::diagnostic::DiagnosticOutput;
use crate::frontend::run_frontend;

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

            let session = Session::new(DiagnosticOutput::stderr());
            let asm = compile(&source, &session)?;

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

fn compile(source: &str, session: &Session) -> CompilerResult<String> {
    let ast_result = run_frontend(session, source);
    let ast = ast_result?;

    Ok(format!("{ast:#?}"))
}
