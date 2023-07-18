use colored::Colorize;

/// Enable the logger
#[allow(dead_code)]
pub fn enable_logger(verbose: bool) {
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
                    match verbose {
                        true => format!(" [{}]", record.target().split("::").nth(0).unwrap()),
                        false => String::new(),
                    }
                    .bright_black()
                ),
                message
            ))
        })
        // Set the correct log level based on CLI flags
        .level(match verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        })
        // Output to STDOUT
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
