mod cli;
mod compilation;

use std::str::FromStr;

use bayou_backend::Linker;
use bayou_session::FullSession;
use bayou_session::FullSessionConfig;
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

    #[error(transparent)]
    InvalidTarget(#[from] target_lexicon::ParseError),

    #[error("backend error: {0}")]
    BackendError(#[from] bayou_backend::BackendError),

    #[error("error writing object: {0}")]
    ObjectError(#[from] bayou_backend::object::write::Error),

    #[error("no linker found for given target and host")]
    NoLinker,

    #[error(transparent)]
    LinkerError(#[from] bayou_backend::LinkerError),

    #[error("errors while compiling")]
    HadErrors,
}

impl From<bayou_session::ErrorsEmitted> for CompilerError {
    fn from(_: bayou_session::ErrorsEmitted) -> Self {
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
            output,
            target,
        } => {
            let name: String = input
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'), "");

            let target = match target {
                Some(s) => Triple::from_str(&s)?,
                None => Triple::host(),
            };

            let linker = Linker::detect(&target).ok_or(CompilerError::NoLinker)?;

            let mut session = FullSession::new(target);

            let output = output.unwrap_or_else(|| name.clone());

            // compilation
            let object = {
                println!("compiling project `{}`", name);

                compile_package(
                    &mut session,
                    FullSessionConfig {
                        name: name.clone(),
                        root_dir: input,
                    },
                )?
            };

            // emit and link objects
            {
                let tmp_dir = TempDir::with_prefix("bayou_")?;

                let tmp_file = TempFileBuilder::new()
                    .in_dir(tmp_dir.path())
                    .prefix(name)
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
