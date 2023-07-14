use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
};

use colored::Colorize;
use ipnet::{Ipv4Net, Ipv6Net};

/// Interface config
#[derive(Debug, serde::Deserialize)]
pub struct InterfaceConfig {
    /// Ipv4 pool
    pub pool: Vec<Ipv4Net>,
    /// IPv6 prefix
    pub prefix: Ipv6Net,
    /// IPv6 router addr
    pub icmpv6_address: Ipv6Addr,
}

/// A static mapping rule
#[derive(Debug, serde::Deserialize)]
pub struct AddressMappingRule {
    /// IPv4 address
    pub v4: Ipv4Addr,
    /// IPv6 address
    pub v6: Ipv6Addr,
}

/// Rules config
#[derive(Debug, serde::Deserialize)]
pub struct RulesConfig {
    /// Static mapping rules
    pub static_map: Vec<AddressMappingRule>,
}

/// Representation of the `protomask.toml` config file
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Interface config
    pub interface: InterfaceConfig,
    /// Rules config
    pub rules: RulesConfig,
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
