use caps::{CapSet, Capability};
use clap::Parser;
use owo_colors::OwoColorize;

mod cli;
mod engines;
mod nat;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = cli::Args::parse();

    // Set up the logger
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}{}: {}",
                // Level messages are padded to keep the output looking somewhat sane
                match record.level() {
                    log::Level::Error => "ERROR"
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.red())
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Warn => "WARN "
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.yellow())
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Info => "INFO "
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.green())
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Debug => "DEBUG"
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bright_blue())
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Trace => "TRACE"
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bright_white())
                        .if_supports_color(owo_colors::Stream::Stdout, |text| text.bold())
                        .to_string(),
                },
                // Only show the outer package name if verbose logging is enabled (otherwise nothing)
                match args.verbose {
                    true => format!(" [{}]", record.target().split("::").next().unwrap()),
                    false => String::new(),
                }
                .if_supports_color(owo_colors::Stream::Stdout, |text| text.bright_black()),
                message
            ))
        })
        // Set the correct log level based on CLI flags
        .level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        })
        // Output to STDOUT
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    // We require NET_ADMIN capabilities to run
    if !caps::has_cap(None, CapSet::Permitted, Capability::CAP_NET_ADMIN).unwrap() {
        log::error!("This program must be run with the NET_ADMIN capability");
        std::process::exit(1);
    }

    // Start up the correct translation engine
    log::info!(
        "Starting translation engine: {}",
        format!("{:?}", args.engine).split(' ').next().unwrap()
    );
    match args.engine {
        cli::Modes::Nat64 {
            interface,
            pool_prefixes,
            static_map,
            translation_prefix,
            lease_duration,
            num_queues,
        } => {
            engines::nat64::do_nat64(
                interface,
                pool_prefixes,
                static_map,
                translation_prefix,
                lease_duration,
                num_queues,
            )
            .await
        }
        cli::Modes::Clat {
            interface,
            customer_pool,
            embed_prefix,
            num_queues,
        } => engines::clat::do_clat(interface, customer_pool, embed_prefix, num_queues).await,
    }

    // We are done at this point
    log::info!("Protomask has finished running. Cleaning up and exiting.");
}
