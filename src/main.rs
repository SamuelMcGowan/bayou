use clap::Parser;
use cli::{Cli, Command};

mod cli;

#[cfg(feature = "test_suite")]
mod test_suite;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
