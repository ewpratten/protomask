//! Commandline arguments and config file definitions for `protomask-clat`

use crate::common::rfc6052::parse_network_specific_prefix;
use ipnet::{Ipv4Net, Ipv6Net};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, clap::Parser)]
#[clap(author, version, about="IPv4 to IPv6 Customer-side transLATor (CLAT)", long_about = None)]
pub struct Args {
    #[command(flatten)]
    config_data: Option<Config>,

    /// Path to a config file to read
    #[clap(short = 'c', long = "config", conflicts_with = "Config")]
    config_file: Option<PathBuf>,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("clat%d").to_string())]
    pub interface: String,

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,
}

impl Args {
    #[allow(dead_code)]
    pub fn data(&self) -> Result<Config, Box<dyn std::error::Error>> {
        match self.config_file {
            Some(ref path) => {
                // Read the data from the config file
                let file = std::fs::File::open(path).map_err(|error| match error.kind() {
                    std::io::ErrorKind::NotFound => {
                        log::error!("Config file not found: {}", path.display());
                        std::process::exit(1)
                    }
                    _ => error,
                })?;
                let data: Config = serde_json::from_reader(file)?;

                // We need at least one customer prefix
                if data.customer_pool.is_empty() {
                    log::error!("No customer prefixes specified. At least one prefix must be specified in the `customer_pool` property of the config file");
                    std::process::exit(1);
                }

                Ok(data)
            }
            None => match &self.config_data {
                Some(data) => Ok(data.clone()),
                None => {
                    log::error!("No configuration provided. Either use --config to specify a file or set the configuration via CLI args (see --help)");
                    std::process::exit(1)
                }
            },
        }
    }
}

/// Program configuration. Specifiable via either CLI args or a config file
#[derive(Debug, clap::Args, serde::Deserialize, Clone)]
#[group()]
pub struct Config {
    /// One or more customer-side IPv4 prefixes to allow through CLAT
    #[clap(long = "customer-prefix")]
    #[serde(rename = "customer_pool")]
    pub customer_pool: Vec<Ipv4Net>,

    /// Enable prometheus metrics on a given address
    #[clap(long = "prometheus")]
    #[serde(rename = "prometheus_bind_addr")]
    pub prom_bind_addr: Option<SocketAddr>,

    /// RFC6052 IPv6 prefix to encapsulate IPv4 packets within
    #[clap(long="via", default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    #[serde(
        rename = "via",
        serialize_with = "crate::common::rfc6052::serialize_network_specific_prefix"
    )]
    pub embed_prefix: Ipv6Net,
}
