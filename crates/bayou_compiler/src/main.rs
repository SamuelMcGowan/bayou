mod compilation;

mod cli;

use std::path::Path;
use std::str::FromStr;

use bayou_backend::linker::{Linker, LinkerError};
use bayou_session::diagnostics::PrettyDiagnosticEmitter;
use bayou_session::sourcemap::Source;
use bayou_session::Session;
use clap::Parser as _;
use cli::{Cli, Command};
use target_lexicon::Triple;
use temp_dir::TempDir;
use temp_file::TempFileBuilder;

use crate::compilation::compile_package;

#[derive(thiserror::Error, Debug)]
enum CompilerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("backend error: {0}")]
    BackendError(#[from] bayou_backend::BackendError),

    #[error(transparent)]
    InvalidTarget(#[from] target_lexicon::ParseError),

    #[error("error writing object: {0}")]
    ObjectError(#[from] bayou_backend::object::write::Error),

    #[error("no linker found for given target and host")]
    NoLinker,

    #[error("linker error: {0}")]
    LinkerError(#[from] LinkerError),

    #[error("errors while compiling")]
    HadErrors,
}

impl From<bayou_session::HadErrors> for CompilerError {
    fn from(_: bayou_session::HadErrors) -> Self {
        Self::HadErrors
    }
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
                Some(s) => Triple::from_str(&s)?,
                None => Triple::host(),
            };

            let linker = Linker::detect(&target).ok_or(CompilerError::NoLinker)?;

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

                let source_id = session.sources.insert(Source { name, source });

                compile_package(&mut session, name_stem.clone(), source_id)?
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
                let object_data = object.write()?;
                std::fs::write(tmp_file.path(), object_data)?;

                println!("linking");
                linker.link(&[tmp_file.path()], output)?;
            }

            Ok(())
        }
    }
}
