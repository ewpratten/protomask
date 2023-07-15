use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
};

use colored::Colorize;
use ipnet::{Ipv4Net, Ipv6Net};

/// Interface config
#[derive(Debug, serde::Deserialize)]
pub struct InterfaceConfig {
    /// IPv4 router address
    #[serde(rename = "Address4")]
    pub address_v4: Ipv4Addr,
    /// IPv6 router address
    #[serde(rename = "Address6")]
    pub address_v6: Ipv6Addr,
    /// Ipv4 pool
    #[serde(rename = "Pool")]
    pub pool: Vec<Ipv4Net>,
    /// IPv6 prefix
    #[serde(rename = "Prefix")]
    pub prefix: Ipv6Net,
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
    #[serde(rename = "MapStatic")]
    pub static_map: Vec<AddressMappingRule>,
}

/// Representation of the `protomask.toml` config file
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    /// Interface config
    #[serde(rename = "Interface")]
    pub interface: InterfaceConfig,
    /// Rules config
    #[serde(rename = "Rules")]
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
