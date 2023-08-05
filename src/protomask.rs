use crate::common::{
    packet_handler::{
        get_ipv4_src_dst, get_ipv6_src_dst, get_layer_3_proto, handle_translation_error,
        PacketHandlingError,
    },
    permissions::ensure_root,
    profiler::start_puffin_server,
};
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

    // Start profiling
    #[allow(clippy::let_unit_value)]
    let _server = start_puffin_server(&args.profiler_args);

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
                    match addr_table.borrow().get_ipv6(&dest) {
                        Some(new_destination) => translate_ipv4_to_ipv6(
                            &buffer[..len],
                            unsafe { embed_ipv4_addr_unchecked(source, config.translation_prefix) },
                            new_destination,
                        )
                        .map(Some)
                        .map_err(PacketHandlingError::from),
                        None => {
                            protomask_metrics::metric!(
                                PACKET_COUNTER,
                                PROTOCOL_IPV4,
                                STATUS_DROPPED
                            );
                            Ok(None)
                        }
                    }
                }
                Some(6) => {
                    let (source, dest) = get_ipv6_src_dst(&buffer[..len]);
                    match addr_table.borrow_mut().get_or_create_ipv4(&source) {
                        Ok(new_source) => {
                            translate_ipv6_to_ipv4(&buffer[..len], new_source, unsafe {
                                extract_ipv4_addr_unchecked(
                                    dest,
                                    config.translation_prefix.prefix_len(),
                                )
                            })
                            .map(Some)
                            .map_err(PacketHandlingError::from)
                        }
                        Err(error) => {
                            log::error!("Error getting IPv4 address: {}", error);
                            protomask_metrics::metric!(
                                PACKET_COUNTER,
                                PROTOCOL_IPV6,
                                STATUS_DROPPED
                            );
                            Ok(None)
                        }
                    }
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
