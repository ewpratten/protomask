use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use ipnet::{Ipv4Net, Ipv6Net};

/// Fast & reliable user space NAT64
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Translation engine
    #[command(subcommand)]
    pub engine: Modes,

    /// Enable verbose logs
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Modes {
    /// Run as a NAT64 translator
    Nat64 {
        /// Explicitly set the interface name to use
        #[clap(short, long, default_value_t = ("nat%d").to_string())]
        interface: String,

        /// IPv4 prefixes to use as NAT pool address space
        #[clap(short, long = "pool-prefix", required = true)]
        pool_prefixes: Vec<Ipv4Net>,

        /// Statically map an IPv4 and IPv6 address to each other
        #[clap(short, long, value_parser = parse_static_map)]
        static_map: Vec<(Ipv4Addr, Ipv6Addr)>,

        /// RFC6052 IPv6 translation prefix
        #[clap(short, long, default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
        translation_prefix: Ipv6Net,

        /// NAT lease duration in seconds
        #[clap(short, long, default_value = "7200")]
        lease_duration: u64,

        /// Number of queues to create on the TUN device
        #[clap(short = 'q', long, default_value = "10")]
        num_queues: usize,
    },

    /// Run as a Customer-side transLATor
    Clat {
        /// Explicitly set the interface name to use
        #[clap(short, long, default_value_t = ("clat%d").to_string())]
        interface: String,

        /// One or more customer-side IPv4 prefixes to allow through CLAT
        #[clap(short, long = "customer-prefix", required = true)]
        customer_pool: Vec<Ipv4Net>,

        /// RFC6052 IPv6 prefix to encapsulate IPv4 packets within
        #[clap(short, long="via", default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
        embed_prefix: Ipv6Net,

        /// Number of queues to create on the TUN device
        #[clap(short = 'q', long, default_value = "10")]
        num_queues: usize,
    },
}

/// Parses an [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)-compliant IPv6 prefix from a string
fn parse_network_specific_prefix(string: &str) -> Result<Ipv6Net, String> {
    // First, parse to an IPv6Net struct
    let net = Ipv6Net::from_str(string).map_err(|err| err.to_string())?;

    // Ensure the prefix length is one of the allowed lengths according to RFC6052 Section 2.2
    if !rfc6052::ALLOWED_PREFIX_LENS.contains(&net.prefix_len()) {
        return Err(format!(
            "Prefix length must be one of {:?}",
            rfc6052::ALLOWED_PREFIX_LENS
        ));
    }

    // Return the parsed network struct
    Ok(net)
}

/// Parses a mapping of an IPv4 address to an IPv6 address
fn parse_static_map(string: &str) -> Result<(Ipv4Addr, Ipv6Addr), String> {
    // Split the string into two parts
    let parts: Vec<&str> = string.split('=').collect();
    if parts.len() != 2 {
        return Err("Static map must be in the form 'IPv4=IPv6'".to_string());
    }

    // Parse the IPv4 and IPv6 addresses
    let v4 = Ipv4Addr::from_str(parts[0]).map_err(|err| err.to_string())?;
    let v6 = Ipv6Addr::from_str(parts[1]).map_err(|err| err.to_string())?;

    // Return the parsed mapping
    Ok((v4, v6))
}