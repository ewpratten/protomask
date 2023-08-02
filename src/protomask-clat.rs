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
use rfc6052::{embed_ipv4_addr_unchecked, extract_ipv4_addr_unchecked};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, Ipv6Addr},
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
            4 => translate_ipv4_to_ipv6(
                &buffer[..len],
                unsafe {
                    embed_ipv4_addr_unchecked(
                        Ipv4Addr::from(u32::from_be_bytes(buffer[12..16].try_into().unwrap())),
                        args.embed_prefix,
                    )
                },
                unsafe {
                    embed_ipv4_addr_unchecked(
                        Ipv4Addr::from(u32::from_be_bytes(buffer[16..20].try_into().unwrap())),
                        args.embed_prefix,
                    )
                },
            ),

            // IPv6
            6 => translate_ipv6_to_ipv4(
                &buffer[..len],
                unsafe {
                    extract_ipv4_addr_unchecked(
                        Ipv6Addr::from(u128::from_be_bytes(buffer[8..24].try_into().unwrap())),
                        args.embed_prefix.prefix_len(),
                    )
                },
                unsafe {
                    extract_ipv4_addr_unchecked(
                        Ipv6Addr::from(u128::from_be_bytes(buffer[24..40].try_into().unwrap())),
                        args.embed_prefix.prefix_len(),
                    )
                },
            ),
            // Unknown
            proto => {
                log::warn!("Unknown Layer 3 protocol: {}", proto);
                continue;
            }
        }
        .unwrap();

        // Write the translated packet back to the TUN interface
        tun.write(&output).unwrap();
    }
}
