//! Entrypoint for the `protomask-clat` binary.
//!
//! This binary is a Customer-side transLATor (CLAT) that translates all native
//! IPv4 traffic to IPv6 traffic for transmission over an IPv6-only ISP network.

use crate::common::packet_handler::{
    get_ipv4_src_dst, get_ipv6_src_dst, get_layer_3_proto, handle_translation_error,
    PacketHandlingError,
};
use crate::common::profiler::start_puffin_server;
use crate::{args::protomask_clat::Args, common::permissions::ensure_root};
use clap::Parser;
use common::logging::enable_logger;
use easy_tun::Tun;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use rfc6052::{embed_ipv4_addr_unchecked, extract_ipv4_addr_unchecked};
use std::io::{Read, Write};

mod args;
mod common;

#[tokio::main]
pub async fn main() {
    // Parse CLI args
    let args = Args::parse();

    // Initialize logging
    enable_logger(args.verbose);

    // Load config data
    let config = args.data().unwrap();

    // We must be root to continue program execution
    ensure_root();

    // Start profiling
    #[allow(clippy::let_unit_value)]
    let _server = start_puffin_server(&args.profiler_args);

    // Bring up a TUN interface
    let mut tun = Tun::new(&args.interface).unwrap();

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
    for customer_prefix in config.customer_pool {
        let embedded_customer_prefix = unsafe {
            Ipv6Net::new(
                embed_ipv4_addr_unchecked(customer_prefix.addr(), config.embed_prefix),
                config.embed_prefix.prefix_len() + customer_prefix.prefix_len(),
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

    // If we are configured to serve prometheus metrics, start the server
    if let Some(bind_addr) = config.prom_bind_addr {
        log::info!("Starting prometheus server on {}", bind_addr);
        tokio::spawn(protomask_metrics::http::serve_metrics(bind_addr));
    }

    // Translate all incoming packets
    log::info!("Translating packets on {}", tun.name());
    let mut buffer = vec![0u8; 1500];
    loop {
        // Indicate to the profiler that we are starting a new packet
        profiling::finish_frame!();
        profiling::scope!("packet");

        // Read a packet
        let len = tun.read(&mut buffer).unwrap();

        // Translate it based on the Layer 3 protocol number
        let translation_result: Result<Option<Vec<u8>>, PacketHandlingError> =
            match get_layer_3_proto(&buffer[..len]) {
                Some(4) => {
                    let (source, dest) = get_ipv4_src_dst(&buffer[..len]);
                    translate_ipv4_to_ipv6(
                        &buffer[..len],
                        unsafe { embed_ipv4_addr_unchecked(source, config.embed_prefix) },
                        unsafe { embed_ipv4_addr_unchecked(dest, config.embed_prefix) },
                    )
                    .map(Some)
                    .map_err(PacketHandlingError::from)
                }
                Some(6) => {
                    let (source, dest) = get_ipv6_src_dst(&buffer[..len]);
                    translate_ipv6_to_ipv4(
                        &buffer[..len],
                        unsafe {
                            extract_ipv4_addr_unchecked(source, config.embed_prefix.prefix_len())
                        },
                        unsafe {
                            extract_ipv4_addr_unchecked(dest, config.embed_prefix.prefix_len())
                        },
                    )
                    .map(Some)
                    .map_err(PacketHandlingError::from)
                }
                Some(proto) => {
                    log::warn!("Unknown Layer 3 protocol: {}", proto);
                    continue;
                }
                None => {
                    continue;
                }
            };

        // Handle any errors and write
        if let Some(output) = handle_translation_error(translation_result) {
            tun.write_all(&output).unwrap();
        }
    }
}
