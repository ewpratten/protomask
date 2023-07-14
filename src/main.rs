use clap::Parser;
use config::Config;

mod config;
mod cli;

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


}
