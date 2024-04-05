#[macro_use]
extern crate macro_rules_attribute;

mod codegen;
mod compilation;
mod ir;
mod parser;
mod passes;
mod platform;
mod resolver;
mod session;
mod symbols;

mod cli;
mod diagnostics;
mod sourcemap;
mod utils;

use std::path::Path;
use std::str::FromStr;

use clap::Parser as _;
use cli::{Cli, Command};
use compilation::PackageCompilation;
use platform::{Linker, LinkerError, PlatformError};
use session::Session;
use target_lexicon::Triple;
use temp_dir::TempDir;
use temp_file::TempFileBuilder;

use crate::diagnostics::PrettyDiagnosticEmitter;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    UnsupportedTarget(#[from] PlatformError),

    #[error(transparent)]
    Module(#[from] cranelift_module::ModuleError),

    #[error(transparent)]
    Codegen(#[from] cranelift::codegen::CodegenError),

    #[error("error emitting object: {0}")]
    Object(#[from] cranelift_object::object::write::Error),

    #[error("linker error: {0}")]
    Linker(#[from] LinkerError),

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
            let target = match target {
                Some(s) => Triple::from_str(&s).map_err(PlatformError::ParseError)?,
                None => Triple::host(),
            };
            let linker = Linker::detect(&target).ok_or(PlatformError::NoLinker)?;

            let mut session = Session::new(target, PrettyDiagnosticEmitter::default());

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

            // compilation
            let object = {
                println!("compiling project `{name}`");

                let pkg = PackageCompilation::start(&mut session, &name, source)?;
                pkg.compile(&mut session)?
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
                linker.link(&[tmp_file.path()], output)?;
            }

            Ok(())
        }
    }
}
