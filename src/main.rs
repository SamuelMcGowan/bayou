use clap::Parser;
use cli::{Cli, Command};
use lexer::Lexer;

#[cfg(feature = "test_suite")]
mod test_suite;

mod ast;
mod cli;
mod lexer;
mod session;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("errors while compiling")]
    HadErrors,
}

type CompilerResult<T> = Result<T, CompilerError>;

fn main() -> CompilerResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build { file } => {
            println!("building file {}", file.display());
            let s = std::fs::read_to_string(file)?;
            println!("s: {s:?}");
        }

        #[cfg(feature = "test_suite")]
        Command::RunTestSuite { stage } => {
            test_suite::run_test_suite(stage)?;
        }
    }

    Ok(())
}

fn compile(source: &str) -> CompilerResult<()> {
    let mut lexer = Lexer::new(source);

    while lexer.lex_token().is_some() {}

    let session = lexer.into_session();

    if session.had_errors() {
        session.flush_diagnostics();
        Err(CompilerError::HadErrors)
    } else {
        Ok(())
    }
}
