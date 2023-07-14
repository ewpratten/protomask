use clap::Parser;
use config::Config;
use nat::Nat64;

mod cli;
mod config;
mod nat;
mod types;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = cli::Args::parse();

    // Set up logging
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("{}: {}", record.level(), message)))
        .level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    if args.verbose {
        log::debug!("Verbose logging enabled");
    }

    // Parse the config file
    let config = Config::load(args.config_file).unwrap();

    // Create the NAT64 instance
    let nat64 = Nat64::new(
        config.interface.address_v4,
        config.interface.address_v6,
        config.interface.pool,
        config.interface.prefix,
        config
            .rules
            .static_map
            .iter()
            .map(|rule| (rule.v4, rule.v6))
            .collect(),
    )
    .await
    .unwrap();

    loop{}
}
