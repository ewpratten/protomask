//! This is the entrypoint for `protomask` from the command line.

use cfg_if::cfg_if;
use clap::Parser;
use config::Config;
use logging::enable_logger;
use protomask::nat::Nat64;

mod cli;
mod config;
mod logging;

#[tokio::main]
pub async fn main() {
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

    // Enable the profiler if enabled by build flags
    cfg_if! {
        if #[cfg(feature = "profiler")] {
            let puffin_listen_addr = format!("[::]:{}", puffin_http::DEFAULT_PORT);
            log::info!("Puffin HTTP server listening on: {}", puffin_listen_addr);
            let _puffin_server = puffin_http::Server::new(&puffin_listen_addr).unwrap();
            puffin::set_scopes_on(true);
        }
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
