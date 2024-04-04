use std::path::Path;
use std::process::Command;

use target_lexicon::{Environment, Triple};

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    #[error(transparent)]
    ParseError(target_lexicon::ParseError),

    #[error("No known linker for given target and host")]
    NoLinker,

    #[error("{0} target unsupported")]
    ArchUnsupported(target_lexicon::Architecture),
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linker {
    Gcc,
}

#[derive(thiserror::Error, Debug)]
pub enum LinkerError {
    #[error("couldn't run linker: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "linker exited with code {} and stderr output {}",
        .code,
        String::from_utf8_lossy(.stderr)
    )]
    Exited { code: i32, stderr: Vec<u8> },

    #[error(
        "linker terminated with stderr output {}",
        String::from_utf8_lossy(.stderr))
    ]
    Terminated { stderr: Vec<u8> },
}

pub struct Platform {
    pub host: Triple,
    pub target: Triple,

    pub linker: Linker,
}

impl Platform {
    pub fn new(target: Triple) -> Result<Self, PlatformError> {
        let host = Triple::host();

        let linker = match (host.environment, target.environment) {
            (Environment::Gnu, Environment::Gnu) => Linker::Gcc,
            _ => return Err(PlatformError::NoLinker),
        };

        Ok(Self {
            host,
            target,
            linker,
        })
    }

    pub fn run_linker(&self, obj_files: &[&Path], output: &str) -> Result<(), LinkerError> {
        let run_linker_command = |mut cmd: Command| -> Result<(), LinkerError> {
            let output = cmd.output()?;
            if output.status.success() {
                Ok(())
            } else {
                match output.status.code() {
                    Some(code) => Err(LinkerError::Exited {
                        code,
                        stderr: output.stderr,
                    }),

                    None => Err(LinkerError::Terminated {
                        stderr: output.stderr,
                    }),
                }
            }
        };

        match self.linker {
            Linker::Gcc => {
                let mut cmd = Command::new("gcc");
                cmd.args(obj_files);
                cmd.arg("-o");
                cmd.arg(output);

                run_linker_command(cmd)
            }
        }
    }
}
