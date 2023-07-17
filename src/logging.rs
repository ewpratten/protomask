use std::sync::OnceLock;

use colored::Colorize;

/// A global variable that is used to early-kill attempts to write debug logs if debug logging is disabled
pub static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

/// A macro that can completely skip the debug step if debug logging is disabled
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if *$crate::logging::DEBUG_ENABLED.get().unwrap_or(&false) {
            log::debug!($($arg)*);
        }
    };
}

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
        .level(match verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    if verbose {
        log::debug!("Verbose logging enabled");
    }

    // Set the global debug enabled variable
    DEBUG_ENABLED.set(verbose).unwrap();
}
