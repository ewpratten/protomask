use crate::{
    count_packet, count_packet_ipv4,
    metrics::{
        labels::PacketStatus,
        registry::{MetricEvent, MetricEventSender},
    },
    packet::{
        protocols::{ipv4::Ipv4Packet, ipv6::Ipv6Packet},
        xlat::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4},
    }, count_packet_ipv6,
};

use self::{
    table::Nat64Table,
    utils::{embed_address, extract_address},
};
use ipnet::{Ipv4Net, Ipv6Net};
use protomask_tun::TunDevice;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use tokio::sync::{broadcast, mpsc};

mod table;
mod utils;

#[derive(Debug, thiserror::Error)]
pub enum Nat64Error {
    #[error(transparent)]
    TableError(#[from] table::TableError),
    #[error(transparent)]
    TunError(#[from] protomask_tun::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    // #[error(transparent)]
    // XlatError(#[from] xlat::PacketTranslationError),
    #[error(transparent)]
    PacketHandlingError(#[from] crate::packet::error::PacketError),
    #[error(transparent)]
    PacketReceiveError(#[from] broadcast::error::RecvError),
    #[error(transparent)]
    PacketSendError(#[from] mpsc::error::SendError<Vec<u8>>),
    #[error(transparent)]
    MetricSendError(#[from] mpsc::error::SendError<MetricEvent>),
}

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
        for ipv4_prefix in ipv4_pool.iter() {
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
    pub async fn run(&mut self, metric_sender: MetricEventSender) -> Result<(), Nat64Error> {
        // Get an rx/tx pair for the interface
        let (tx, mut rx) = self.interface.spawn_worker().await;

        // Process packets in a loop
        loop {
            // Try to read a packet
            match rx.recv().await {
                Ok(packet) => {
                    // Clone the TX so the worker can respond with data
                    let tx = tx.clone();

                    // Separate logic is needed for handling IPv4 vs IPv6 packets, so a check must be done here
                    match packet[0] >> 4 {
                        4 => {
                            // Parse the packet
                            let packet: Ipv4Packet<Vec<u8>> = packet.try_into()?;

                            // Drop packets that aren't destined for a destination the table knows about
                            if !self.table.contains(&IpAddr::V4(packet.destination_address)) {
                                // Update metrics
                                count_packet_ipv4!(metric_sender, PacketStatus::Dropped)?;

                                // Drop packet
                                continue;
                            }

                            // Get the new source and dest addresses
                            let new_source =
                                embed_address(packet.source_address, self.ipv6_nat_prefix);
                            let new_destination =
                                self.table.get_reverse(packet.destination_address)?;

                            // Spawn a task to process the packet
                            let metric_sender = metric_sender.clone();
                            tokio::spawn(async move {
                                count_packet_ipv4!(metric_sender, PacketStatus::Accepted).unwrap();

                                // Process the packet
                                let output =
                                    translate_ipv4_to_ipv6(packet, new_source, new_destination)
                                        .unwrap();

                                // Send the translated packet
                                tx.send(output.into()).await.unwrap();
                                count_packet_ipv6!(metric_sender, PacketStatus::Sent).unwrap();
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
                                count_packet_ipv6!(metric_sender, PacketStatus::Dropped)?;
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
                                count_packet_ipv6!(metric_sender, PacketStatus::Dropped)?;
                                continue;
                            }

                            // Spawn a task to process the packet
                            let metric_sender = metric_sender.clone();
                            tokio::spawn(async move {
                                count_packet_ipv6!(metric_sender, PacketStatus::Accepted).unwrap();
                                let output =
                                    translate_ipv6_to_ipv4(packet, new_source, new_destination)
                                        .unwrap();
                                tx.send(output.into()).await.unwrap();
                                count_packet_ipv4!(metric_sender, PacketStatus::Sent).unwrap();
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
                    error => Err(error),
                },
            }?;
        }
    }
}
