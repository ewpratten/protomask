#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod metrics;
pub mod nat;
mod packet;

use clap::Parser;
use config::Config;
use logging::enable_logger;

mod cli;
mod config;
mod logging;

async fn run_nat(config_file: PathBuf) {
// Parse the config file
let config = Config::load(args.config_file).unwrap();

// Currently, only a /96 is supported
if config.nat64_prefix.prefix_len() != 96 {
    log::error!("Only a /96 prefix is supported for the NAT64 prefix");
    std::process::exit(1);
}

// Create the NAT64 instance
let mut nat64 = Nat64::new(
    config.nat64_prefix,
    config.pool.prefixes.clone(),
    config
        .pool
        .static_map
        .iter()
        .map(|rule| (rule.v6, rule.v4))
        .collect(),
    config.pool.reservation_duration(),
)
.await
.unwrap();



// Handle packets
nat64.run().await.unwrap();
}

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = cli::Args::parse();

    // Set up logging
    enable_logger(args.verbose);

    // Handle metrics requests
if let Some(bind_addr) = config.prom_bind_addr {
    log::info!("Enabling metrics server on {}", bind_addr);
    tokio::spawn(protomask::metrics::serve_metrics(bind_addr));
}

    
}
