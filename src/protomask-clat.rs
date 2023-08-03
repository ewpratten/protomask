//! Entrypoint for the `protomask-clat` binary.
//!
//! This binary is a Customer-side transLATor (CLAT) that translates all native
//! IPv4 traffic to IPv6 traffic for transmission over an IPv6-only ISP network.

use clap::Parser;
use common::{logging::enable_logger, rfc6052::parse_network_specific_prefix};
use easy_tun::Tun;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
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
    log::debug!("Creating new TUN interface");
    let mut tun = Tun::new(&args.interface).unwrap();
    log::debug!("Created TUN interface: {}", tun.name());

    // Configure the new interface
    // - Bring up
    // - Add IPv6 prefix as a route
    // - Point IPv4 default route to the new interface
    let rt_handle = rtnl::new_handle().unwrap();
    let tun_link_idx = rtnl::link::get_link_index(&rt_handle, tun.name())
        .await
        .unwrap()
        .unwrap();
    rtnl::link::link_up(&rt_handle, tun_link_idx).await.unwrap();
    rtnl::route::route_add(IpNet::V6(args.embed_prefix), &rt_handle, tun_link_idx)
        .await
        .unwrap();
    rtnl::route::route_add(IpNet::V4(Ipv4Net::default()), &rt_handle, tun_link_idx)
        .await
        .unwrap();

    // Translate all incoming packets
    log::info!("Translating packets on {}", tun.name());
    let mut buffer = vec![0u8; 1500];
    loop {
        // Read a packet
        let len = tun.read(&mut buffer).unwrap();

        // Translate it based on the Layer 3 protocol number
        let layer_3_proto = buffer[0] >> 4;
        log::trace!("New packet with layer 3 protocol: {}", layer_3_proto);
        match match layer_3_proto {
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
        } {
            Ok(data) => {
                // Write the translated packet back to the TUN interface
                tun.write(&data).unwrap();
            }
            Err(error) => match error {
                interproto::error::Error::PacketTooShort { expected, actual } => log::warn!(
                    "Got packet with length {} when expecting at least {} bytes",
                    actual,
                    expected
                ),
                interproto::error::Error::UnsupportedIcmpType(icmp_type) => {
                    log::warn!("Got a packet with an unsupported ICMP type: {}", icmp_type)
                }
                interproto::error::Error::UnsupportedIcmpv6Type(icmpv6_type) => log::warn!(
                    "Got a packet with an unsupported ICMPv6 type: {}",
                    icmpv6_type
                ),
            },
        };
    }
}
