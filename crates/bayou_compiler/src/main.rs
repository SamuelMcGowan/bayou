#[macro_use]
extern crate macro_rules_attribute;

mod codegen;
mod compiler;
mod ir;
mod parser;
mod passes;
mod resolver;
mod symbols;
mod target;

mod cli;
mod diagnostics;
mod sourcemap;
mod utils;

use std::path::Path;
use std::str::FromStr;

use clap::Parser as _;
use cli::{Cli, Command};
use target::UnsupportedTarget;
use target_lexicon::Triple;
use temp_dir::TempDir;
use temp_file::TempFileBuilder;

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
            let (name, name_stem, source) = if source {
                ("unnamed".to_owned(), "unnamed".to_owned(), input)
            } else {
                let source = std::fs::read_to_string(&input)?;
                let name_normalised = Path::new(&input)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                (input, name_normalised, source)
            };
            let output = output.unwrap_or_else(|| name_stem.clone());

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

                let tmp_file = TempFileBuilder::new()
                    .in_dir(tmp_dir.path())
                    .prefix(name_stem)
                    .suffix(".o")
                    .build()?;

                println!("writing object");
                let object_data = object.emit()?;
                std::fs::write(tmp_file.path(), object_data)?;

                println!("linking");
                linker.run(&[tmp_file.path()], &output)?;
            }

            Ok(())
        }
    }
}
