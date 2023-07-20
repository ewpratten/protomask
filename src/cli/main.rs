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

    // Enable Sentry reporting
    cfg_if! {
        if #[cfg(feature = "sentry")] {
            log::debug!("Enabling Sentry reporting");
            let _guard = sentry::init(("https://376a29d0fd7c40d0a82f05a7e8c3600e@o4504175421947904.ingest.sentry.io/4505563054080000", sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            }));
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

    // Handle metrics requests
    if let Some(bind_addr) = config.prom_bind_addr {
        log::info!("Enabling metrics server on {}", bind_addr);
        tokio::spawn(protomask::metrics::serve_metrics(bind_addr));
    }

    // Handle packets
    nat64.run().await.unwrap();
}
