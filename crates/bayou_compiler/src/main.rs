#[macro_use]
extern crate macro_rules_attribute;

mod frontend;

mod cli;
mod diagnostic;
mod ir;
mod symbols;
mod utils;

use clap::Parser as _;
use cli::{Cli, Command};

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
            let source = std::fs::read_to_string(&input)?;

            let asm = compile(input.to_string_lossy(), source)?;

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

fn compile(source_name: impl Into<String>, source: impl Into<String>) -> CompilerResult<String> {
    let source = source.into();
    // let mut sources = session.sources.borrow_mut();

    // let source_id = sources.len();
    // sources.push(Cached::new((source_name.into(), source.into())));

    // let source = sources.get(source_id).unwrap().source_str().to_owned();

    // drop(sources);

    let ast_result = run_frontend(&source);
    let ast = ast_result?;

    Ok(format!("{ast:#?}"))
}
