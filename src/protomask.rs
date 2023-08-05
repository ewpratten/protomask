use crate::common::{packet_handler::handle_packet, permissions::ensure_root};
use clap::Parser;
use common::logging::enable_logger;
use easy_tun::Tun;
use fast_nat::CrossProtocolNetworkAddressTableWithIpv4Pool;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::IpNet;
use rfc6052::{embed_ipv4_addr_unchecked, extract_ipv4_addr_unchecked};
use std::{
    cell::RefCell,
    io::{Read, Write},
    time::Duration,
};

mod args;
mod common;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = args::protomask::Args::parse();

    // Initialize logging
    enable_logger(args.verbose);

    // Load config data
    let config = args.data().unwrap();

    // We must be root to continue program execution
    ensure_root();

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

    // Add a route for the translation prefix
    log::debug!(
        "Adding route for {} to {}",
        config.translation_prefix,
        tun.name()
    );
    rtnl::route::route_add(
        IpNet::V6(config.translation_prefix),
        &rt_handle,
        tun_link_idx,
    )
    .await
    .unwrap();

    // Add a route for each NAT pool prefix
    for pool_prefix in &config.pool_prefixes {
        log::debug!("Adding route for {} to {}", pool_prefix, tun.name());
        rtnl::route::route_add(IpNet::V4(*pool_prefix), &rt_handle, tun_link_idx)
            .await
            .unwrap();
    }

    // Set up the address table
    let mut addr_table = RefCell::new(CrossProtocolNetworkAddressTableWithIpv4Pool::new(
        &config.pool_prefixes,
        Duration::from_secs(config.reservation_timeout),
    ));
    for (v4_addr, v6_addr) in &config.static_map {
        addr_table
            .get_mut()
            .insert_static(*v4_addr, *v6_addr)
            .unwrap();
    }

    // If we are configured to serve prometheus metrics, start the server
    if let Some(bind_addr) = config.prom_bind_addr {
        log::info!("Starting prometheus server on {}", bind_addr);
        tokio::spawn(protomask_metrics::http::serve_metrics(bind_addr));
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
            |packet, source, dest| match addr_table.borrow().get_ipv6(dest) {
                Some(new_destination) => Ok(translate_ipv4_to_ipv6(
                    packet,
                    unsafe { embed_ipv4_addr_unchecked(*source, config.translation_prefix) },
                    new_destination,
                )
                .map(Some)?),
                None => {
                    protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_IPV4, STATUS_DROPPED);
                    Ok(None)
                }
            },
            // IPv6 -> IPv4
            |packet, source, dest| {
                Ok(translate_ipv6_to_ipv4(
                    packet,
                    addr_table.borrow_mut().get_or_create_ipv4(source)?,
                    unsafe {
                        extract_ipv4_addr_unchecked(*dest, config.translation_prefix.prefix_len())
                    },
                )
                .map(Some)?)
            },
        ) {
            // Write the packet if we get one back from the handler functions
            tun.write_all(&output).unwrap();
        }
    }
}
