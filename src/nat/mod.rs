use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use ipnet::{Ipv4Net, Ipv6Net};
use pnet_packet::{ip::IpNextHeaderProtocols, Packet};

use crate::{into_tcp, into_udp, ipv4_packet, ipv6_packet, nat::xlat::translate_udp_4_to_6};

use self::{
    interface::Nat64Interface,
    packet::IpPacket,
    table::{Nat64Table, TableError},
};

mod interface;
mod macros;
mod packet;
mod table;
mod xlat;

#[derive(Debug, thiserror::Error)]
pub enum Nat64Error {
    #[error(transparent)]
    TableError(#[from] table::TableError),
    #[error(transparent)]
    InterfaceError(#[from] interface::InterfaceError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XlatError(#[from] xlat::PacketTranslationError),
}

pub struct Nat64 {
    table: Nat64Table,
    interface: Nat64Interface,
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
        let interface = Nat64Interface::new(ipv6_nat_prefix, &ipv4_pool).await?;

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
        // Allocate a buffer for incoming packets
        let mut buffer = vec![0u8; self.interface.mtu()];

        // Loop forever
        loop {
            // Read a packet from the interface
            match self.interface.recv(&mut buffer) {
                Ok(packet_len) => {
                    // Parse in to a more friendly format
                    match IpPacket::new(&buffer[..packet_len]) {
                        // Try to process the packet
                        Ok(inbound_packet) => match self.process_packet(inbound_packet).await {
                            Ok(inbound_packet) => match inbound_packet {
                                // If data is returned, send it back out the interface
                                Some(outbound_packet) => {
                                    let packet_bytes = outbound_packet.to_bytes();
                                    log::debug!(
                                        "Outbound packet next header: {}",
                                        outbound_packet.get_next_header().0
                                    );
                                    log::debug!("Sending packet: {:?}", packet_bytes);
                                    self.interface.send(&packet_bytes).unwrap();
                                }
                                // Otherwise, we can assume that the packet was dealt with, and can move on
                                None => {}
                            },

                            // Some errors are non-critical as far as this loop is concerned
                            Err(error) => match error {
                                Nat64Error::TableError(TableError::NoIpv6Mapping(address)) => {
                                    log::debug!("No IPv6 mapping for {}", address);
                                }
                                error => {
                                    return Err(error);
                                }
                            },
                        },
                        Err(error) => {
                            log::error!("Failed to parse packet: {}", error);
                        }
                    }
                }
                Err(error) => {
                    log::error!("Failed to read packet: {}", error);
                }
            }
        }
    }
}

impl Nat64 {
    async fn process_packet<'a>(
        &mut self,
        packet: IpPacket<'a>,
    ) -> Result<Option<IpPacket<'a>>, Nat64Error> {
        // The destination of the packet must be within a prefix we care about
        if match packet.get_destination() {
            IpAddr::V4(ipv4_addr) => !self.table.is_address_within_pool(&ipv4_addr),
            IpAddr::V6(ipv6_addr) => !self.ipv6_nat_prefix.contains(&ipv6_addr),
        } {
            log::debug!(
                "Packet destination {} is not within the NAT64 prefix or IPv4 pool",
                packet.get_destination(),
            );
            return Ok(None);
        }

        // Compute the translated source and dest addresses
        let source = packet.get_source();
        let new_source = self
            .table
            .calculate_xlat_addr(&source, &self.ipv6_nat_prefix)?;
        let destination = packet.get_destination();
        let new_destination = self
            .table
            .calculate_xlat_addr(&destination, &self.ipv6_nat_prefix)?;

        // Log information about the packet
        log::debug!(
            "Received packet traveling from {} to {}",
            source,
            destination
        );
        log::debug!(
            "New path shall become: {} -> {}",
            new_source,
            new_destination
        );

        // Different logic is required for ICMP, UDP, and TCP
        match (packet, new_source, new_destination) {
            (IpPacket::V4(packet), IpAddr::V6(new_source), IpAddr::V6(new_destination)) => {
                match packet.get_next_level_protocol() {
                    // User Datagram Protocol
                    IpNextHeaderProtocols::Udp => Ok(Some(IpPacket::V6(ipv6_packet!(
                        new_source,
                        new_destination,
                        IpNextHeaderProtocols::Udp,
                        packet.get_ttl(),
                        translate_udp_4_to_6(
                            into_udp!(packet.payload().to_vec())?,
                            new_source,
                            new_destination
                        )?
                        .packet()
                    )))),

                    // Transmission Control Protocol
                    IpNextHeaderProtocols::Tcp => Ok(Some(IpPacket::V6(ipv6_packet!(
                        new_source,
                        new_destination,
                        IpNextHeaderProtocols::Tcp,
                        packet.get_ttl(),
                        xlat::translate_tcp_4_to_6(
                            into_tcp!(packet.payload().to_vec())?,
                            new_source,
                            new_destination
                        )?
                        .packet()
                    )))),

                    // For any protocol we don't support, just warn and drop the packet
                    next_level_protocol => {
                        log::warn!("Unsupported next level protocol: {}", next_level_protocol);
                        Ok(None)
                    }
                }
            }
            (IpPacket::V6(packet), IpAddr::V4(new_source), IpAddr::V4(new_destination)) => {
                match packet.get_next_header() {
                    // User Datagram Protocol
                    IpNextHeaderProtocols::Udp => Ok(Some(IpPacket::V4(ipv4_packet!(
                        new_source,
                        new_destination,
                        packet.get_hop_limit(),
                        IpNextHeaderProtocols::Udp,
                        xlat::translate_udp_6_to_4(
                            into_udp!(packet.payload().to_vec())?,
                            new_source,
                            new_destination
                        )?
                        .packet()
                    )))),

                    // Transmission Control Protocol
                    IpNextHeaderProtocols::Tcp => Ok(Some(IpPacket::V4(ipv4_packet!(
                        new_source,
                        new_destination,
                        packet.get_hop_limit(),
                        IpNextHeaderProtocols::Tcp,
                        xlat::translate_tcp_6_to_4(
                            into_tcp!(packet.payload().to_vec())?,
                            new_source,
                            new_destination
                        )?
                        .packet()
                    )))),

                    // For any protocol we don't support, just warn and drop the packet
                    next_header_protocol => {
                        log::warn!("Unsupported next header protocol: {}", next_header_protocol);
                        Ok(None)
                    }
                }
            }

            // Honestly, this should probably be `unreachable!()`
            _ => unimplemented!(),
        }
        // match next_header_protocol {
        //     IpNextHeaderProtocols::Icmp | IpNextHeaderProtocols::Icmpv6 => Ok(
        //         xlat::proxy_icmp_packet(packet, new_source, new_destination)?,
        //     ),
        //     IpNextHeaderProtocols::Udp => Ok(Some(
        //         xlat::proxy_udp_packet(packet, new_source, new_destination).await?,
        //     )),
        //     IpNextHeaderProtocols::Tcp => Ok(Some(
        //         xlat::proxy_tcp_packet(packet, new_source, new_destination).await?,
        //     )),
        //     next_header_protocol => {
        //         log::warn!("Unsupported next header protocol: {}", next_header_protocol);
        //         Ok(None)
        //     }
        // }
    }
}
