use clap::Parser;
use colored::Colorize;
use config::Config;

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
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}: {}",
                match record.level() {
                    log::Level::Error => "ERROR".red().bold().to_string(),
                    log::Level::Warn => "WARN".yellow().bold().to_string(),
                    log::Level::Info => "INFO".green().bold().to_string(),
                    log::Level::Debug => "DEBUG".bright_blue().bold().to_string(),
                    log::Level::Trace => "TRACE".bright_white().bold().to_string(),
                },
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

    // Parse the config file
    let config = Config::load(args.config_file).unwrap();

    // // Create the NAT64 instance
    // let mut nat64 = Nat64::new(
    //     config.interface.pool,
    //     config.interface.prefix,
    //     config
    //         .rules
    //         .static_map
    //         .iter()
    //         .map(|rule| (rule.v4, rule.v6))
    //         .collect(),
    // )
    // .await
    // .unwrap();

    // // Handle packets
    // nat64.run().await.unwrap();
}
