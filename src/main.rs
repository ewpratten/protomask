//! The `protomask` application entrypoint

#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use clap::Parser;
use cli::{config::Config, logging::enable_logger};
use nat::Nat64;

mod cli;
mod metrics;
mod nat;
mod packet;
mod tun;

#[tokio::main]
pub async fn main() {
    // Enable profiling server if we are building with `--features profiling`
    #[cfg(feature = "profiling")]
    let _puffin_server =
        puffin_http::Server::new(&format!("[::]:{}", puffin_http::DEFAULT_PORT)).unwrap();
    #[cfg(feature = "profiling")]
    puffin::set_scopes_on(true);

    // Parse CLI args
    let args = cli::Args::parse();

    // Set up logging
    enable_logger(args.verbose);

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

    // Handle metrics requests
    if let Some(bind_addr) = config.prom_bind_addr {
        log::info!("Enabling metrics server on {}", bind_addr);
        tokio::spawn(metrics::serve_metrics(bind_addr));
    }

    // Handle packets
    nat64.run().await.unwrap();
}
