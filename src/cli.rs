use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {

    /// Path to the config file
    pub config_file: PathBuf,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,
}
