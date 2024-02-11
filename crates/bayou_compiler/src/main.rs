#[macro_use]
extern crate macro_rules_attribute;

mod parser;

mod cli;
mod codegen;
mod compiler;
mod diagnostics;
mod ir;
mod resolver;
mod sourcemap;
mod symbols;
mod target;
mod utils;

use std::str::FromStr;

use clap::Parser as _;
use cli::{Cli, Command};
use target::UnsupportedTarget;
use target_lexicon::Triple;
use temp_dir::TempDir;

use crate::compiler::Compiler;
use crate::diagnostics::PrettyDiagnosticEmitter;
use crate::target::Linker;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    UnsupportedTarget(#[from] UnsupportedTarget),

    #[error(transparent)]
    Module(#[from] cranelift_module::ModuleError),

    #[error(transparent)]
    Codegen(#[from] cranelift::codegen::CodegenError),

    #[error("error emitting object: {0}")]
    Object(#[from] cranelift_object::object::write::Error),

    #[error("linker error: {}", String::from_utf8_lossy(.0))]
    Linker(Vec<u8>),

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
        Command::Build {
            input,
            source,
            output,
            target,
        } => {
            let (name, source) = if source {
                ("unnamed".to_owned(), input)
            } else {
                let source = std::fs::read_to_string(&input)?;
                (input, source)
            };
            let output = output.unwrap_or_else(|| name.clone());

            let triple = match target {
                Some(s) => Triple::from_str(&s).map_err(UnsupportedTarget::ParseError)?,
                None => Triple::host(),
            };
            let linker = Linker::from_triple(&triple)?;

            // compilation
            let object = {
                println!("compiling project `{name}`");

                let mut compiler = Compiler::new(
                    name.clone(),
                    PrettyDiagnosticEmitter::default(),
                    triple.clone(),
                );

                let _module_id = compiler.add_module(&name, source)?;
                compiler.compile()?
            };

            // emit and link objects
            {
                let tmp_dir = TempDir::with_prefix("bayou_")?;

                let object_path = tmp_dir.path().join(&name).with_extension("o");

                println!("writing object");
                let object_data = object.emit()?;
                std::fs::write(&object_path, object_data)?;

                println!("linking");
                linker.run(&[object_path.as_path()], &output)?;
            }

            Ok(())
        }
    }
}
