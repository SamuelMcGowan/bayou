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
        /// The input directory.
        input: PathBuf,

        /// The output file. If not specified, prints assembly to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// The target triple.
        #[arg(short, long)]
        target: Option<String>,
    },
}
