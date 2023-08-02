//! Command line argument definitions

use std::{net::Ipv6Addr, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use ipnet::{Ipv4Net, Ipv6Net};

/// Shorthand for generating the well-known NAT64 prefix
macro_rules! wkp {
    () => {
        Ipv6Net::new(
            Ipv6Addr::new(0x0064, 0xff9b, 0x000, 0x0000, 0x000, 0x0000, 0x000, 0x0000),
            96,
        )
        .unwrap()
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable OpenMetrics on a given address
    pub openmetrics_addr: 

    /// Enable verbose logging
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run protomask in NAT64 mode
    Nat64 {
        /// IPv6 prefix to listen for packets on
        #[clap(short='l', long = "listen", default_value_t = wkp!(), value_parser = nat64_prefix_parser)]
        listen_prefix: Ipv6Net,

        /// Add an IPv4 prefix to the NAT pool
        #[clap(long = "nat", required = true)]
        nat_pool: Vec<Ipv4Net>,
    },
    /// Run protomask in Customer-side transLATor (CLAT) mode
    ///
    /// CLAT mode will translate all native IPv4 traffic to IPv6 traffic.
    Clat {
        /// IPv6 prefix to use for source addressing
        #[clap(long = "via", default_value_t = wkp!(), value_parser = nat64_prefix_parser)]
        origin_prefix: Ipv6Net,
    },
    /// Run protomask in RFC2529 / 6over4 mode
    SixOverFour {
        /// The IPv4 network interface to communicate over
        #[clap()]
        interface: String,
    }
}

/// Parses an IPv6 prefix and ensures it is at most a /96
fn nat64_prefix_parser(s: &str) -> Result<Ipv6Net, String> {
    let net = Ipv6Net::from_str(s).map_err(|err| err.to_string())?;
    if net.prefix_len() > 96 {
        return Err("Prefix length must be 96 or less".to_owned());
    }
    Ok(net)
} 

