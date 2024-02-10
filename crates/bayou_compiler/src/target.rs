use std::path::Path;
use std::process::Command;

use target_lexicon::{Environment, Triple};

use crate::{CompilerError, CompilerResult};

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linker {
    Gcc,
}

impl Linker {
    pub fn from_triple(triple: &Triple) -> Result<Self, UnsupportedTarget> {
        match triple.environment {
            Environment::Gnu => Ok(Linker::Gcc),
            _ => Err(UnsupportedTarget::NoLinkerFound(triple.clone())),
        }
    }
}

pub fn run_linker(linker: Linker, obj_files: &[&Path], output: &str) -> CompilerResult<()> {
    match linker {
        Linker::Gcc => {
            let mut cmd = Command::new("gcc");
            cmd.args(obj_files);
            cmd.arg("-o");
            cmd.arg(output);

            let output = cmd.output()?;
            if output.status.success() {
                Ok(())
            } else {
                Err(CompilerError::Linker(output.stderr))
            }
        }
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedTarget {
    #[error(transparent)]
    ParseError(target_lexicon::ParseError),

    #[error("No linker found for target {0}")]
    NoLinkerFound(Triple),

    #[error("{0} unsupported")]
    ArchUnsupported(target_lexicon::Architecture),
}
