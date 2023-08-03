use std::path::PathBuf;

use clap::Parser;
use common::{logging::enable_logger, rfc6052::parse_network_specific_prefix};
use ipnet::{Ipv4Net, Ipv6Net};
use nix::unistd::Uid;

mod common;

#[derive(Parser)]
#[clap(author, version, about="Fast and simple NAT64", long_about = None)]
struct Args {
    /// RFC6052 IPv6 translation prefix
    #[clap(long, default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    translation_prefix: Ipv6Net,

    #[command(flatten)]
    pool: PoolArgs,

    /// A CSV file containing static address mappings from IPv6 to IPv4
    #[clap(long = "static-file")]
    static_file: Option<PathBuf>,

    /// NAT reservation timeout in seconds
    #[clap(long, default_value = "7200")]
    reservation_timeout: u64,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("nat%d").to_string())]
    interface: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct PoolArgs {
    /// IPv4 prefixes to use as NAT pool address space
    #[clap(long = "pool-add")]
    pool_prefixes: Vec<Ipv4Net>,

    /// A file containing newline-delimited IPv4 prefixes to use as NAT pool address space
    #[clap(long = "pool-file", conflicts_with = "pool_prefixes")]
    pool_file: Option<PathBuf>,
}

impl PoolArgs {
    pub fn prefixes(&self) -> Result<Vec<Ipv4Net>, std::io::Error> {
        todo!()
    }
}

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = Args::parse();

    // Initialize logging
    enable_logger(args.verbose);

    // We must be root to continue program execution
    if !Uid::effective().is_root() {
        log::error!("This program must be run as root");
        std::process::exit(1);
    }
}
