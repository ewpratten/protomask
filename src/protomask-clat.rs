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

use crate::common::packet_handler::handle_packet;

mod common;

#[derive(Debug, Parser)]
#[clap(author, version, about="IPv4 to IPv6 Customer-side transLATor (CLAT)", long_about = None)]
struct Args {
    /// IPv6 prefix to embed IPv4 addresses in
    #[clap(long="via", default_value_t = ("64:ff9b::/96").parse().unwrap(), value_parser = parse_network_specific_prefix)]
    embed_prefix: Ipv6Net,

    /// One or more customer-side IPv4 prefixes to allow through CLAT
    #[clap(short = 'c', long = "customer-prefix", required = true)]
    customer_pool: Vec<Ipv4Net>,

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

    // Get the interface index
    let rt_handle = rtnl::new_handle().unwrap();
    let tun_link_idx = rtnl::link::get_link_index(&rt_handle, tun.name())
        .await
        .unwrap()
        .unwrap();

    // Bring the interface up
    rtnl::link::link_up(&rt_handle, tun_link_idx).await.unwrap();

    // Add an IPv4 default route towards the interface
    rtnl::route::route_add(IpNet::V4(Ipv4Net::default()), &rt_handle, tun_link_idx)
        .await
        .unwrap();

    // Add an IPv6 route for each customer prefix
    for customer_prefix in args.customer_pool {
        let embedded_customer_prefix = unsafe {
            Ipv6Net::new(
                embed_ipv4_addr_unchecked(customer_prefix.addr(), args.embed_prefix),
                args.embed_prefix.prefix_len() + customer_prefix.prefix_len(),
            )
            .unwrap_unchecked()
        };
        log::debug!(
            "Adding route for {} to {}",
            embedded_customer_prefix,
            tun.name()
        );
        rtnl::route::route_add(
            IpNet::V6(embedded_customer_prefix),
            &rt_handle,
            tun_link_idx,
        )
        .await
        .unwrap();
    }

    // Translate all incoming packets
    log::info!("Translating packets on {}", tun.name());
    let mut buffer = vec![0u8; 1500];
    loop {
        // Read a packet
        let len = tun.read(&mut buffer).unwrap();

        // Translate it based on the Layer 3 protocol number
        if let Some(output) = handle_packet(
            &buffer[..len],
            // IPv4 -> IPv6
            |packet, source, dest| {
                translate_ipv4_to_ipv6(
                    packet,
                    unsafe { embed_ipv4_addr_unchecked(*source, args.embed_prefix) },
                    unsafe { embed_ipv4_addr_unchecked(*dest, args.embed_prefix) },
                )
            },
            // IPv6 -> IPv4
            |packet, source, dest| {
                translate_ipv6_to_ipv4(
                    packet,
                    unsafe { extract_ipv4_addr_unchecked(*source, args.embed_prefix.prefix_len()) },
                    unsafe { extract_ipv4_addr_unchecked(*dest, args.embed_prefix.prefix_len()) },
                )
            },
        ) {
            // Write the packet if we get one back from the handler functions
            tun.write(&output).unwrap();
        }
    }
}
