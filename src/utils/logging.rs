use owo_colors::{OwoColorize, Stream::Stdout};

/// Enable the logger
#[allow(dead_code)]
pub fn enable_logger(verbose: bool) {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}{}: {}",
                // Level messages are padded to keep the output looking somewhat sane
                match record.level() {
                    log::Level::Error => "ERROR"
                        .if_supports_color(Stdout, |text| text.red())
                        .if_supports_color(Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Warn => "WARN "
                        .if_supports_color(Stdout, |text| text.yellow())
                        .if_supports_color(Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Info => "INFO "
                        .if_supports_color(Stdout, |text| text.green())
                        .if_supports_color(Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Debug => "DEBUG"
                        .if_supports_color(Stdout, |text| text.bright_blue())
                        .if_supports_color(Stdout, |text| text.bold())
                        .to_string(),
                    log::Level::Trace => "TRACE"
                        .if_supports_color(Stdout, |text| text.bright_white())
                        .if_supports_color(Stdout, |text| text.bold())
                        .to_string(),
                },
                // Only show the outer package name if verbose logging is enabled (otherwise nothing)
                match verbose {
                    true => format!(" [{}]", record.target().split("::").next().unwrap()),
                    false => String::new(),
                }
                .if_supports_color(Stdout, |text| text.bright_black()),
                message
            ))
        })
        // Set the correct log level based on CLI flags
        .level(match std::env::var("PROTOMASK_TRACE") {
            Ok(_) => log::LevelFilter::Trace,
            Err(_) => match verbose {
                true => log::LevelFilter::Debug,
                false => log::LevelFilter::Info,
            },
        })
        // Output to STDOUT
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
