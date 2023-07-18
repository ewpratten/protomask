use std::net::{IpAddr, SocketAddr};

use pnet_packet::Packet;

use crate::packet::error::PacketError;

/// A UDP packet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UdpPacket<T> {
    source: SocketAddr,
    destination: SocketAddr,
    pub payload: T,
}

impl<T> UdpPacket<T> {
    /// Construct a new UDP packet
    pub fn new(
        source: SocketAddr,
        destination: SocketAddr,
        payload: T,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source.is_ipv4() != destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source.ip(),
                destination.ip(),
            ));
        }

        // Build the packet
        Ok(Self {
            source,
            destination,
            payload,
        })
    }

    // Set a new source
    pub fn set_source(&mut self, source: SocketAddr) -> Result<(), PacketError> {
        // Ensure the source and destination addresses are the same type
        if source.is_ipv4() != self.destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source.ip(),
                self.destination.ip(),
            ));
        }

        // Set the source
        self.source = source;

        Ok(())
    }

    // Set a new destination
    pub fn set_destination(&mut self, destination: SocketAddr) -> Result<(), PacketError> {
        // Ensure the source and destination addresses are the same type
        if self.source.is_ipv4() != destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                self.source.ip(),
                destination.ip(),
            ));
        }

        // Set the destination
        self.destination = destination;

        Ok(())
    }

    /// Get the source
    pub fn source(&self) -> SocketAddr {
        self.source
    }

    /// Get the destination
    pub fn destination(&self) -> SocketAddr {
        self.destination
    }
}

impl<T> UdpPacket<T>
where
    T: From<Vec<u8>>,
{
    /// Construct a new UDP packet from bytes
    pub fn new_from_bytes(
        bytes: &[u8],
        source_address: IpAddr,
        destination_address: IpAddr,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source_address.is_ipv4() != destination_address.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source_address,
                destination_address,
            ));
        }

        // Parse the packet
        let parsed = pnet_packet::udp::UdpPacket::new(bytes)
            .ok_or_else(|| PacketError::TooShort(bytes.len()))?;

        // Build the struct
        Ok(Self {
            source: SocketAddr::new(source_address, parsed.get_source()),
            destination: SocketAddr::new(destination_address, parsed.get_destination()),
            payload: parsed.payload().to_vec().into(),
        })
    }
}

impl UdpPacket<Vec<u8>> {
    /// Construct a new UDP packet with a raw payload from bytes
    pub fn new_from_bytes_raw_payload(
        bytes: &[u8],
        source_address: IpAddr,
        destination_address: IpAddr,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source_address.is_ipv4() != destination_address.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source_address,
                destination_address,
            ));
        }

        // Parse the packet
        let parsed = pnet_packet::udp::UdpPacket::new(bytes)
            .ok_or_else(|| PacketError::TooShort(bytes.len()))?;

        // Build the struct
        Ok(Self {
            source: SocketAddr::new(source_address, parsed.get_source()),
            destination: SocketAddr::new(destination_address, parsed.get_destination()),
            payload: parsed.payload().to_vec(),
        })
    }
}

impl<T> Into<Vec<u8>> for UdpPacket<T>
where
    T: Into<Vec<u8>>,
{
    fn into(self) -> Vec<u8> {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = self.payload.into();

        // Allocate a mutable packet to write into
        let total_length =
            pnet_packet::udp::MutableUdpPacket::minimum_packet_size() + payload.len();
        let mut output =
            pnet_packet::udp::MutableUdpPacket::owned(vec![0u8; total_length]).unwrap();

        // Write the source and dest ports
        output.set_source(self.source.port());
        output.set_destination(self.destination.port());

        // Write the length
        output.set_length(total_length as u16);

        // Write the payload
        output.set_payload(&payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(match (self.source.ip(), self.destination.ip()) {
            (IpAddr::V4(source_ip), IpAddr::V4(destination_ip)) => {
                pnet_packet::udp::ipv4_checksum(&output.to_immutable(), &source_ip, &destination_ip)
            }
            (IpAddr::V6(source_ip), IpAddr::V6(destination_ip)) => {
                pnet_packet::udp::ipv6_checksum(&output.to_immutable(), &source_ip, &destination_ip)
            }
            _ => unreachable!(),
        });

        // Return the raw bytes
        output.packet().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test packet construction
    #[test]
    #[rustfmt::skip]
    fn test_packet_construction() {
        // Make a new packet
        let packet = UdpPacket::new(
            "192.0.2.1:1234".parse().unwrap(),
            "192.0.2.2:5678".parse().unwrap(),
            "Hello, world!".as_bytes().to_vec(),
        )
        .unwrap();

        // Convert to raw bytes
        let packet_bytes: Vec<u8> = packet.into();

        // Check the contents
        assert!(packet_bytes.len() >= 8 + 13);
        assert_eq!(u16::from_be_bytes([packet_bytes[0], packet_bytes[1]]), 1234);
        assert_eq!(u16::from_be_bytes([packet_bytes[2], packet_bytes[3]]), 5678);
        assert_eq!(u16::from_be_bytes([packet_bytes[4], packet_bytes[5]]), 8 + 13);
        assert_eq!(u16::from_be_bytes([packet_bytes[6], packet_bytes[7]]), 0x1f74);
        assert_eq!(
            &packet_bytes[8..],
            "Hello, world!".as_bytes().to_vec().as_slice()
        );
    }
}
