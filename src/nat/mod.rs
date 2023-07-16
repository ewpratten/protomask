use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use ipnet::{Ipv4Net, Ipv6Net};

use self::{interface::Nat64Interface, packet::IpPacket, table::Nat64Table};

mod interface;
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
}

pub struct Nat64 {
    table: Nat64Table,
    interface: Nat64Interface,
}

impl Nat64 {
    /// Construct a new NAT64 instance
    pub async fn new(
        v6_prefix: Ipv6Net,
        v4_pool: Vec<Ipv4Net>,
        static_reservations: Vec<(Ipv6Addr, Ipv4Addr)>,
        reservation_duration: Duration,
    ) -> Result<Self, Nat64Error> {
        // Bring up the interface
        let interface = Nat64Interface::new(v6_prefix, &v4_pool).await?;

        // Build the table and insert any static reservations
        let mut table = Nat64Table::new(v4_pool, reservation_duration);
        for (v6, v4) in static_reservations {
            table.add_infinite_reservation(v6, v4)?;
        }

        Ok(Self { table, interface })
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
                        Ok(inbound_packet) => match self.process_packet(inbound_packet).await? {
                            // If data is returned, send it back out the interface
                            Some(outbound_packet) => {
                                let packet_bytes = outbound_packet.to_bytes();
                                self.interface.send(&packet_bytes)?;
                            }
                            // Otherwise, we can assume that the packet was dealt with, and can move on
                            None => {}
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
    async fn process_packet(
        &mut self,
        packet: IpPacket<'_>,
    ) -> Result<Option<IpPacket>, Nat64Error> {
        Ok(None)
    }
}
