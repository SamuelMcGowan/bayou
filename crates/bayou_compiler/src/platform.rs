use std::ffi::OsStr;
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

#[derive(thiserror::Error, Debug)]
pub enum LinkerError {
    #[error("couldn't run linker: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "linker exited with code {code} and stderr output {}",
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
    pub target: Triple,
    pub linker: Linker,
}

impl Platform {
    pub fn new(
        target: Triple,
        custom_linker: Option<(String, Vec<String>)>,
    ) -> Result<Self, PlatformError> {
        let linker = custom_linker
            .map(|(cmd, args)| Linker::Custom(cmd, args))
            .or_else(|| Linker::detect(&target))
            .ok_or(PlatformError::NoLinker)?;

        Ok(Self { target, linker })
    }
}

pub enum Linker {
    Gcc,
    Custom(String, Vec<String>),
}

impl Linker {
    pub fn detect(target: &Triple) -> Option<Self> {
        let host = Triple::host();

        if target != &host {
            return None;
        }

        if host.environment == Environment::Gnu {
            Some(Self::Gcc)
        } else {
            None
        }
    }

    pub fn link<P0: AsRef<OsStr>, P1: AsRef<OsStr>>(
        &self,
        obj_files: &[P0],
        output: P1,
    ) -> Result<(), LinkerError> {
        let mut cmd = match self {
            Self::Gcc => {
                let mut cmd = Command::new("gcc");

                cmd.arg("-o");
                cmd.arg(output);

                cmd.args(obj_files);

                cmd
            }

            Self::Custom(cmd, args) => {
                let mut cmd = Command::new(cmd);
                cmd.args(args);

                cmd.arg("-o");
                cmd.arg(output);

                cmd.args(obj_files);

                cmd
            }
        };

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
    }
}
