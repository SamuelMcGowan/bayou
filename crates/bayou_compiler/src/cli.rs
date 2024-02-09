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
    /// Build a program.
    Build {
        /// The input file.
        input: String,

        /// Whether the given input should be used directly as the source instead
        /// of as the source file path.
        #[clap(long, short, action)]
        source: bool,

        /// The output file. If not specified, prints assembly to stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
