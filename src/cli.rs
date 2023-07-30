//! Command line argument definitions

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Nat64 {
        /// Path to the config file
        config_file: PathBuf,
    },
    Clat {},
}
