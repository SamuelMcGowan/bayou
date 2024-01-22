use clap::Parser as _;
use cli::{Cli, Command};
use codegen::CodeGenerator;
use parser::Parser;
use session::Session;

#[cfg(test)]
mod tests;

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

    let parser = Parser::new(&session, source);
    let module = parser.parse_module();

    if session.had_errors() {
        if print_diagnostics {
            session.flush_diagnostics();
        }
        return Err(CompilerError::HadErrors);
    }

    let codegen = CodeGenerator::new(&session);
    let asm = codegen.run(&module);

    Ok(asm)
}
