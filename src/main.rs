use clap::Parser as _;
use cli::{Cli, Command};
use parser::Parser;
use session::Session;

#[cfg(feature = "test_suite")]
mod test_suite;

mod ast;
mod cli;
mod lexer;
mod parser;
mod session;

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
        Command::Build { file } => {
            println!("building file {}", file.display());
            let s = std::fs::read_to_string(file)?;
            compile(&s, true)
        }

        #[cfg(feature = "test_suite")]
        Command::RunTestSuite { stage } => test_suite::run_test_suite(stage),
    }
}

fn compile(source: &str, print_diagnostics: bool) -> CompilerResult<()> {
    let session = Session::default();

    let parser = Parser::new(&session, source);

    let _module = parser.parse_module();

    if session.had_errors() {
        if print_diagnostics {
            session.flush_diagnostics();
        }
        Err(CompilerError::HadErrors)
    } else {
        Ok(())
    }
}
