use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
    time::Duration,
};

use colored::Colorize;
use ipnet::{Ipv4Net, Ipv6Net};

/// A static mapping rule
#[derive(Debug, serde::Deserialize)]
pub struct AddressMappingRule {
    /// IPv4 address
    pub v4: Ipv4Addr,
    /// IPv6 address
    pub v6: Ipv6Addr,
}

/// Used to generate the default reservation duration
fn default_reservation_duration() -> u64 {
    7200
}

/// Rules config
#[derive(Debug, serde::Deserialize)]
pub struct PoolConfig {
    /// Pool prefixes
    #[serde(rename = "Prefixes")]
    pub prefixes: Vec<Ipv4Net>,
    /// Static mapping rules
    #[serde(rename = "Static", default = "Vec::new")]
    pub static_map: Vec<AddressMappingRule>,
    /// How long to hold a dynamic mapping for
    #[serde(rename = "MaxIdleDuration", default = "default_reservation_duration")]
    reservation_duration: u64,
}

impl PoolConfig {
    /// Get the reservation duration
    pub fn reservation_duration(&self) -> Duration {
        Duration::from_secs(self.reservation_duration)
    }
}

/// Representation of the `protomask.toml` config file
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// The NAT64 prefix
    #[serde(rename = "Nat64Prefix")]
    pub nat64_prefix: Ipv6Net,
    /// Pool configuration
    #[serde(rename = "Pool")]
    pub pool: PoolConfig,
}

impl Config {
    /// Load the config from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        // Load the file
        let file_contents = std::fs::read_to_string(path)?;

        // Build the deserializer
        let deserializer = toml::Deserializer::new(&file_contents);

        // Parse
        match serde_path_to_error::deserialize(deserializer) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!(
                    "Failed to parse config file due to:\n {}\n at {}",
                    e.inner().message().bright_red(),
                    e.path().to_string().bright_cyan()
                );
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that fails if the example file is not valid
    #[test]
    fn ensure_example_is_valid() {
        let _ = Config::load("protomask.toml").unwrap();
    }
}
