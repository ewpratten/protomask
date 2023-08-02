//! Entrypoint for the `protomask-clat` binary.
//!
//! This binary is a Customer-side transLATor (CLAT) that translates all native
//! IPv4 traffic to IPv6 traffic for transmission over an IPv6-only ISP network.

use clap::Parser;
use common::{logging::enable_logger, rfc6052::parse_network_specific_prefix};
use easy_tun::Tun;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::Ipv6Net;
use nix::unistd::Uid;
use std::{
    io::{Read, Write},
    net::Ipv4Addr,
};

mod common;

#[derive(Debug, Parser)]
#[clap(author, version, about="IPv4 to IPv6 Customer-side transLATor (CLAT)", long_about = None)]
struct Args {
    /// IPv6 prefix to embed IPv4 addresses in
    #[clap(long="via", default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    embed_prefix: Ipv6Net,

    /// Explicitly set the interface name to use
    #[clap(short, long, default_value_t = ("clat%d").to_string())]
    interface: String,

    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = Args::parse();

    // Initialize logging
    enable_logger(args.verbose);

    // We must be root to continue program execution
    if !Uid::effective().is_root() {
        log::error!("This program must be run as root");
        std::process::exit(1);
    }

    // Bring up a TUN interface
    let mut tun = Tun::new(&args.interface).unwrap();

    // Translate all incoming packets
    log::info!("Translating packets on {}", tun.name());
    let mut buffer = vec![0u8; 1500];
    loop {
        // Read a packet
        let len = tun.read(&mut buffer).unwrap();

        // Translate it based on the Layer 3 protocol number
        let layer_3_proto = buffer[0] >> 4;
        log::trace!("New packet with layer 3 protocol: {}", layer_3_proto);
        let output = match layer_3_proto {
            // IPv4
            4 => {
                // Get the IPv4 source and destination addresses
                let ipv4_source =
                    u32::from_be_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]);
                let ipv4_destination =
                    u32::from_be_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]);

                // Create a new IPv6 source and destination address by embedding the IPv4 addresses into the clat prefix
                let new_source = u128::from(args.embed_prefix.addr()) | (ipv4_source as u128);
                let new_destination =
                    u128::from(args.embed_prefix.addr()) | (ipv4_destination as u128);

                translate_ipv4_to_ipv6(&buffer[..len], new_source.into(), new_destination.into())
            }

            // IPv6
            6 => translate_ipv6_to_ipv4(
                &buffer[..len],
                // NOTE: The new source and destination addresses are just the last
                // 4 octets of the IPv6 source and destination addresses
                Ipv4Addr::new(buffer[20], buffer[21], buffer[22], buffer[23]),
                Ipv4Addr::new(buffer[36], buffer[37], buffer[38], buffer[39]),
            ),
            // Unknown
            proto => {
                log::warn!("Unknown Layer 3 protocol: {}", proto);
                continue;
            }
        }
        .unwrap();
    }
}
