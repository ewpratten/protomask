use clap::Parser;
use colored::Colorize;
use config::Config;
use nat::Nat64;

mod cli;
mod config;
mod nat;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = cli::Args::parse();

    // Set up logging
    let log_verbose = args.verbose;
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}: {}",
                format!(
                    "{}{}",
                    // Level messages are padded to keep the output looking somewhat sane
                    match record.level() {
                        log::Level::Error => "ERROR".red().bold().to_string(),
                        log::Level::Warn => "WARN ".yellow().bold().to_string(),
                        log::Level::Info => "INFO ".green().bold().to_string(),
                        log::Level::Debug => "DEBUG".bright_blue().bold().to_string(),
                        log::Level::Trace => "TRACE".bright_white().bold().to_string(),
                    },
                    // Only show the outer package name if verbose logging is enabled (otherwise nothing)
                    match log_verbose {
                        true => format!(" [{}]", record.target().split("::").nth(0).unwrap()),
                        false => String::new(),
                    }
                    .bright_black()
                ),
                message
            ))
        })
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
