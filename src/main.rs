use clap::Parser as _;
use cli::{Cli, Command};
use session::Session;

use crate::backend::CodeGenerator;
use crate::frontend::parse_and_build_ir;

mod backend;
mod frontend;
mod ir;

mod cli;
mod session;
mod symbols;
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

fn compile(source: &str, print_diagnostics: bool) -> CompilerResult<String> {
    let session = Session::default();

    let (ir, symbols) = parse_and_build_ir(&session, source).map_err(|err| {
        if let CompilerError::HadErrors = err {
            if print_diagnostics {
                session.flush_diagnostics();
            }
        }
        err
    })?;

    let codegen = CodeGenerator::new(&session, symbols);
    let asm = codegen.run(&ir);

    Ok(asm)
}
