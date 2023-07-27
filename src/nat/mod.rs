use crate::{
    metrics::PACKET_COUNTER,
    packet::{
        protocols::{ipv4::Ipv4Packet, ipv6::Ipv6Packet},
        xlat::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4},
    }, profiling::PacketTimer,
};

use self::{
    error::Nat64Error,
    table::Nat64Table,
    utils::{embed_address, extract_address, unwrap_log},
};
use ipnet::{Ipv4Net, Ipv6Net};
use protomask_tun::TunDevice;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use tokio::sync::broadcast;

mod error;
mod table;
mod utils;

pub struct Nat64 {
    table: Nat64Table,
    interface: TunDevice,
    ipv6_nat_prefix: Ipv6Net,
}

impl Nat64 {
    /// Construct a new NAT64 instance
    pub async fn new(
        ipv6_nat_prefix: Ipv6Net,
        ipv4_pool: Vec<Ipv4Net>,
        static_reservations: Vec<(Ipv6Addr, Ipv4Addr)>,
        reservation_duration: Duration,
    ) -> Result<Self, Nat64Error> {
        // Bring up the interface
        let mut interface = TunDevice::new("nat64i%d").await?;

        // Add the NAT64 prefix as a route
        interface.add_route(ipv6_nat_prefix.into()).await?;

        // Add the IPv4 pool prefixes as routes
        for ipv4_prefix in &ipv4_pool {
            interface.add_route((*ipv4_prefix).into()).await?;
        }

        // Build the table and insert any static reservations
        let mut table = Nat64Table::new(ipv4_pool, reservation_duration);
        for (v6, v4) in static_reservations {
            table.add_infinite_reservation(v6, v4)?;
        }

        Ok(Self {
            table,
            interface,
            ipv6_nat_prefix,
        })
    }

    /// Block and process all packets
    pub async fn run(&mut self) -> Result<(), Nat64Error> {
        // Get an rx/tx pair for the interface
        let (tx, mut rx) = self.interface.spawn_worker().await;

        // Only test if we should be printing profiling data once. This won't change mid-execution
        let should_print_profiling = std::env::var("PROTOMASK_TRACE").is_ok();

        // Process packets in a loop
        loop {
            // Try to read a packet
            match rx.recv().await {
                Ok(packet) => {
                    // Clone the TX so the worker can respond with data
                    let tx = tx.clone();

                    // Build a profiling object.
                    // This will be used by various functions to keep rough track of
                    // how long each major operation takes in the lifecycle of this Packet
                    let mut timer = PacketTimer::new(packet[0] >> 4);

                    // Separate logic is needed for handling IPv4 vs IPv6 packets, so a check must be done here
                    match packet[0] >> 4 {
                        4 => {

                            // Parse the packet
                            let packet: Ipv4Packet<Vec<u8>> = packet.try_into()?;

                            // Drop packets that aren't destined for a destination the table knows about
                            if !self.table.contains(&IpAddr::V4(packet.destination_address)) {
                                PACKET_COUNTER.with_label_values(&["ipv4", "dropped"]).inc();
                                continue;
                            }

                            // Get the new source and dest addresses
                            let new_source =
                                embed_address(packet.source_address, self.ipv6_nat_prefix);
                            let new_destination =
                                self.table.get_reverse(packet.destination_address)?;

                            // Mark the packet as accepted
                            PACKET_COUNTER
                                .with_label_values(&["ipv4", "accepted"])
                                .inc();

                            // Spawn a task to process the packet
                            tokio::spawn(async move {
                                if let Some(output) = unwrap_log(translate_ipv4_to_ipv6(
                                    packet,
                                    new_source,
                                    new_destination,
                                    &mut timer,
                                )) {
                                    tx.send(output.into()).await.unwrap();
                                    if should_print_profiling {
                                        timer.log();
                                    }
                                    PACKET_COUNTER.with_label_values(&["ipv6", "sent"]).inc();
                                }
                            });
                        }
                        6 => {

                            // Parse the packet
                            let packet: Ipv6Packet<Vec<u8>> = packet.try_into()?;

                            // Drop packets "coming from" the NAT64 prefix
                            if self.ipv6_nat_prefix.contains(&packet.source_address) {
                                log::warn!(
                                    "Dropping packet \"from\" NAT64 prefix: {} -> {}",
                                    packet.source_address,
                                    packet.destination_address
                                );
                                PACKET_COUNTER.with_label_values(&["ipv6", "dropped"]).inc();
                                continue;
                            }

                            // Get the new source and dest addresses
                            let new_source =
                                self.table.get_or_assign_ipv4(packet.source_address)?;
                            let new_destination = extract_address(packet.destination_address);

                            // Drop packets destined for private IPv4 addresses
                            if new_destination.is_private() {
                                log::warn!(
                                    "Dropping packet destined for private IPv4 address: {} -> {} ({})",
                                    packet.source_address,
                                    packet.destination_address,
                                    new_destination
                                );
                                PACKET_COUNTER.with_label_values(&["ipv6", "dropped"]).inc();
                                continue;
                            }

                            // Mark the packet as accepted
                            PACKET_COUNTER
                                .with_label_values(&["ipv6", "accepted"])
                                .inc();

                            // Spawn a task to process the packet
                            tokio::spawn(async move {
                                if let Some(output) = unwrap_log(translate_ipv6_to_ipv4(
                                    &packet,
                                    new_source,
                                    new_destination,
                                    &mut timer,
                                )) {
                                    tx.send(output.into()).await.unwrap();
                                    if should_print_profiling {
                                        timer.log();
                                    }
                                    PACKET_COUNTER.with_label_values(&["ipv4", "sent"]).inc();
                                }
                            });
                        }
                        n => {
                            log::warn!("Unknown IP version: {}", n);
                        }
                    }
                    Ok(())
                }
                Err(error) => match error {
                    broadcast::error::RecvError::Lagged(count) => {
                        log::warn!("Translator running behind! Dropping {} packets", count);
                        Ok(())
                    }
                    error @ broadcast::error::RecvError::Closed => Err(error),
                },
            }?;
        }
    }
}
