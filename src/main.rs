use clap::Parser as _;
use cli::{Cli, Command};
use codegen::CodeGenerator;
use parser::Parser;
use session::Session;

#[cfg(feature = "test_suite")]
mod test_suite;

#[cfg(test)]
mod tests2;

mod ast;
mod cli;
mod codegen;
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

fn compile(source: &str, show_output: bool) -> CompilerResult<()> {
    let session = Session::default();

    let parser = Parser::new(&session, source);
    let module = parser.parse_module();

    if session.had_errors() {
        if show_output {
            session.flush_diagnostics();
        }
        return Err(CompilerError::HadErrors);
    }

    let codegen = CodeGenerator::new(&session);
    let asm = codegen.run(&module);

    if show_output {
        println!("ASSEMBLY:\n\n{asm}");
    }

    Ok(())
}
