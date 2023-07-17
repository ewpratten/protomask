use clap::Parser;
use config::Config;
use logging::enable_logger;
use nat::Nat64;

mod cli;
mod config;
mod nat;
mod logging;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = cli::Args::parse();

    // Set up logging
    enable_logger(args.verbose);

    // If the binary was built with profiling support, enable it
    #[cfg(feature = "enable-profiling")]
    let _puffin_server: puffin_http::Server;
    #[cfg(feature = "enable-profiling")]
    if args.enable_profiling {
        _puffin_server =
            puffin_http::Server::new(&format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT)).unwrap();
        log::info!("Puffin profiling server started");
    }

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
