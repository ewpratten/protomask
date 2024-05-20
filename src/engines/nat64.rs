use easy_tun::Tun;
use interproto::protocols::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use rfc6052::{embed_ipv4_addr_unchecked, extract_ipv4_addr_unchecked};
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::nat::NetworkAddressTranslationTable;

/// Run NAT64 logic
pub async fn do_nat64(
    interface: String,
    pool_prefixes: Vec<Ipv4Net>,
    static_map: Vec<(Ipv4Addr, Ipv6Addr)>,
    translation_prefix: Ipv6Net,
    lease_duration: u64,
    num_queues: usize,
) {
    // Create a TUN interface
    let tun = Arc::new(
        Tun::new(&interface, num_queues)
            .map_err(|err| {
                log::error!("Failed to create TUN interface");
                err
            })
            .unwrap(),
    );

    // Get the "interface index" of our new interface
    let rt_netlink_handle = rtnl::new_handle().unwrap();
    let tun_link_index = rtnl::link::get_link_index(&rt_netlink_handle, tun.name())
        .await
        .unwrap()
        .unwrap();
    log::debug!("TUN interface index: {}", tun_link_index);

    // Bring the interface up
    log::info!("Bringing up TUN interface: {}", tun.name());
    rtnl::link::link_up(&rt_netlink_handle, tun_link_index)
        .await
        .unwrap();

    // Add a route for the translation prefix
    log::debug!("Adding route for {} to {}", translation_prefix, tun.name());
    rtnl::route::route_add(
        IpNet::V6(translation_prefix),
        &rt_netlink_handle,
        tun_link_index,
    )
    .await
    .unwrap();

    // Add a route for each pool prefix
    for pool_prefix in &pool_prefixes {
        log::debug!("Adding route for {} to {}", pool_prefix, tun.name());
        rtnl::route::route_add(
            IpNet::V4(pool_prefix.clone()),
            &rt_netlink_handle,
            tun_link_index,
        )
        .await
        .unwrap();
    }

    // Set up the address table
    let address_table = Arc::new(Mutex::new(NetworkAddressTranslationTable::new(
        Duration::from_secs(lease_duration),
    )));
    for (ipv4, ipv6) in static_map {
        log::info!("Adding static mapping: {} <--> {}", ipv4, ipv6);
        address_table
            .lock()
            .unwrap()
            .add_pair(ipv4, ipv6, false)
            .unwrap();
    }

    // Perform translation
    log::info!("Starting {} worker threads...", num_queues);
    let mut worker_threads = Vec::new();
    for queue_id in 0..num_queues {
        let tun = tun.clone();
        let address_table = address_table.clone();
        let pool_prefixes = pool_prefixes.clone();
        worker_threads.push(thread::spawn(move || {
            log::debug!("Starting worker thread for queue ID: {}", queue_id);

            // Allocate a buffer for the packet
            // TODO: Add custom MTU support
            let mut buffer = vec![0u8; 1500];

            // Process forever
            loop {
                // Read a packet
                let len = tun.fd(queue_id).unwrap().read(&mut buffer).unwrap();
                let packet = &buffer[..len];

                // If we have an empty packet, skip
                if len == 0 {
                    log::debug!("Skipping empty packet");
                    continue;
                }

                // Determine the layer 3 protocol
                let layer_3_proto = packet[0] >> 4;
                log::trace!("Handling packet with layer 3 protocol: {}", layer_3_proto);

                // Try to translate the packet
                match layer_3_proto {
                    // Ipv4 -> IPv6
                    4 => {
                        // Figure out the original (untranslated) source and destination addresses
                        let ipv4_source_addr =
                            Ipv4Addr::from(u32::from_be_bytes(packet[12..16].try_into().unwrap()));
                        let ipv4_destination_addr =
                            Ipv4Addr::from(u32::from_be_bytes(packet[16..20].try_into().unwrap()));

                        // Look up the appropriate IPv6 address for the destination
                        let ipv6_destination_addr = address_table
                            .lock()
                            .unwrap()
                            .get_ipv6(&ipv4_destination_addr);

                        // If we have a mapping for the destination address, we can translate the packet
                        if let Some(ipv6_destination_addr) = ipv6_destination_addr {
                            // Construct a new IPv6 source addr
                            let ipv6_source_addr = unsafe {
                                embed_ipv4_addr_unchecked(ipv4_source_addr, translation_prefix)
                            };

                            if let Ok(translated_packet) = translate_ipv4_to_ipv6(
                                packet,
                                ipv6_source_addr,
                                ipv6_destination_addr,
                            ) {
                                // Update the lease
                                address_table
                                    .lock()
                                    .unwrap()
                                    .tick_flow(&IpAddr::V4(ipv4_destination_addr));

                                // Write the translated packet to the TUN interface
                                tun.fd(queue_id)
                                    .unwrap()
                                    .write_all(&translated_packet)
                                    .unwrap();
                            }
                        }
                    }

                    // IPv6 -> IPv4
                    6 => {
                        // Figure out the original (untranslated) source and destination addresses
                        let ipv6_source_addr =
                            Ipv6Addr::from(u128::from_be_bytes(packet[8..24].try_into().unwrap()));
                        let ipv6_destination_addr =
                            Ipv6Addr::from(u128::from_be_bytes(packet[24..40].try_into().unwrap()));

                        // Check if we have a source IPv4 address already leased. If not, attempt to allocate one
                        let mut ipv4_source_addr =
                            address_table.lock().unwrap().get_ipv4(&ipv6_source_addr);
                        if ipv4_source_addr.is_none() {
                            // Find the next available IPv4 address in the pool
                            ipv4_source_addr =
                                address_table.lock().unwrap().find_free_ipv4(&pool_prefixes);

                            // Try to insert a new mapping for this address
                            if let Some(ipv4_source_addr) = ipv4_source_addr {
                                address_table
                                    .lock()
                                    .unwrap()
                                    .add_pair(ipv4_source_addr, ipv6_source_addr, true)
                                    .unwrap();
                                log::debug!(
                                    "Created mapping for {} <--> {}",
                                    ipv6_source_addr,
                                    ipv4_source_addr
                                );
                            } else {
                                // If we are here, we have run out of IPv4 addresses
                                log::warn!(
                                    "IPv4 pool exhausted. {} did not get a lease",
                                    ipv6_source_addr
                                );
                                continue;
                            }
                        }

                        // Unwrap the source address
                        let ipv4_source_addr = ipv4_source_addr.unwrap();

                        // Figure out the correct IPv4 destination address
                        let ipv4_destination_addr = unsafe {
                            extract_ipv4_addr_unchecked(
                                ipv6_destination_addr,
                                translation_prefix.prefix_len(),
                            )
                        };

                        // Translate the packet
                        if let Ok(translated_packet) =
                            translate_ipv6_to_ipv4(packet, ipv4_source_addr, ipv4_destination_addr)
                        {
                            // Update the lease
                            address_table
                                .lock()
                                .unwrap()
                                .tick_flow(&IpAddr::V6(ipv6_source_addr));

                            // Write the translated packet to the TUN interface
                            tun.fd(queue_id)
                                .unwrap()
                                .write_all(&translated_packet)
                                .unwrap();
                        }
                    }

                    // Unknown protocol
                    _ => {
                        log::warn!("Unsupported layer 3 protocol: {}", layer_3_proto);
                        continue;
                    }
                }
            }
        }));
    }

    // Spawn a helper thread that periodically cleans up expired leases
    let address_table = address_table.clone();
    worker_threads.push(thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(30));
        log::trace!("Pruning expired leases");
        address_table.lock().unwrap().prune();
    }));

    // Wait for all workers to finish
    log::info!("Processing packets");
    for worker in worker_threads {
        worker.join().unwrap();
    }
}
