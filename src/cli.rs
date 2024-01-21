use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Build {
        file: PathBuf,
    },

    #[cfg(feature = "test_suite")]
    RunTestSuite {
        stage: usize,
    },
}
