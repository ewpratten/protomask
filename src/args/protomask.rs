use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

use ipnet::{Ipv4Net, Ipv6Net};

use crate::common::rfc6052::parse_network_specific_prefix;

use super::ProfilerArgs;

#[derive(clap::Parser)]
#[clap(author, version, about="Fast and simple NAT64", long_about = None)]
pub struct Args {
    #[command(flatten)]
    config_data: Option<Config>,

    /// Path to a config file to read
    #[clap(short = 'c', long = "config", conflicts_with = "Config")]
    config_file: Option<PathBuf>,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("nat%d").to_string())]
    pub interface: String,

    #[command(flatten)]
    pub profiler_args: ProfilerArgs,

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

                // We need at least one pool prefix
                if data.pool_prefixes.is_empty() {
                    log::error!("No pool prefixes specified. At least one prefix must be specified in the `pool` property of the config file");
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
    /// IPv4 prefixes to use as NAT pool address space
    #[clap(long = "pool-prefix")]
    #[serde(rename = "pool")]
    pub pool_prefixes: Vec<Ipv4Net>,

    /// Static mapping between IPv4 and IPv6 addresses
    #[clap(skip)]
    pub static_map: Vec<(Ipv4Addr, Ipv6Addr)>,

    /// Enable prometheus metrics on a given address
    #[clap(long = "prometheus")]
    #[serde(rename = "prometheus_bind_addr")]
    pub prom_bind_addr: Option<SocketAddr>,

    /// RFC6052 IPv6 translation prefix
    #[clap(long, default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    #[serde(
        rename = "prefix",
        serialize_with = "crate::common::rfc6052::serialize_network_specific_prefix"
    )]
    pub translation_prefix: Ipv6Net,

    /// NAT reservation timeout in seconds
    #[clap(long, default_value = "7200")]
    pub reservation_timeout: u64,

    /// Number of queues to create on the TUN device
    #[clap(long, default_value = "10")]
    #[serde(rename = "queues")]
    pub num_queues: usize,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct StaticMap {
    pub ipv4: Ipv4Addr,
    pub ipv6: Ipv6Addr,
}

impl From<StaticMap> for (Ipv4Addr, Ipv6Addr) {
    fn from(val: StaticMap) -> Self {
        (val.ipv4, val.ipv6)
    }
}
