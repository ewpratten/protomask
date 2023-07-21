use std::net::Ipv6Addr;

use pnet_packet::{
    icmpv6::{Icmpv6Code, Icmpv6Type},
    Packet,
};

use crate::packet::error::PacketError;

use super::raw::RawBytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Icmpv6Packet<T> {
    pub source_address: Ipv6Addr,
    pub destination_address: Ipv6Addr,
    pub icmp_type: Icmpv6Type,
    pub icmp_code: Icmpv6Code,
    pub payload: T,
}

impl<T> Icmpv6Packet<T> {
    /// Construct a new ICMPv6 packet
    pub fn new(
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
        icmp_type: Icmpv6Type,
        icmp_code: Icmpv6Code,
        payload: T,
    ) -> Self {
        Self {
            source_address,
            destination_address,
            icmp_type,
            icmp_code,
            payload,
        }
    }
}

impl<T> Icmpv6Packet<T>
where
    T: From<Vec<u8>>,
{
    /// Construct a new ICMPv6 packet from raw bytes
    #[allow(dead_code)]
    pub fn new_from_bytes(
        bytes: &[u8],
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
    ) -> Result<Self, PacketError> {
        // Parse the packet
        let packet = pnet_packet::icmpv6::Icmpv6Packet::new(bytes)
            .ok_or(PacketError::TooShort(bytes.len(), bytes.to_vec()))?;

        // Return the packet
        Ok(Self {
            source_address,
            destination_address,
            icmp_type: packet.get_icmpv6_type(),
            icmp_code: packet.get_icmpv6_code(),
            payload: packet.payload().to_vec().into(),
        })
    }
}

impl Icmpv6Packet<RawBytes> {
    /// Construct a new ICMPv6 packet with a raw payload from raw bytes
    pub fn new_from_bytes_raw_payload(
        bytes: &[u8],
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
    ) -> Result<Self, PacketError> {
        // Parse the packet
        let packet = pnet_packet::icmpv6::Icmpv6Packet::new(bytes)
            .ok_or(PacketError::TooShort(bytes.len(), bytes.to_vec()))?;

        // Return the packet
        Ok(Self {
            source_address,
            destination_address,
            icmp_type: packet.get_icmpv6_type(),
            icmp_code: packet.get_icmpv6_code(),
            payload: RawBytes(packet.payload().to_vec()),
        })
    }
}

impl<T> From<Icmpv6Packet<T>> for Vec<u8>
where
    T: Into<Vec<u8>>,
{
    fn from(packet: Icmpv6Packet<T>) -> Self {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = packet.payload.into();

        // Allocate a mutable packet to write into
        let total_length =
            pnet_packet::icmpv6::MutableIcmpv6Packet::minimum_packet_size() + payload.len();
        let mut output =
            pnet_packet::icmpv6::MutableIcmpv6Packet::owned(vec![0u8; total_length]).unwrap();

        // Write the type and code
        output.set_icmpv6_type(packet.icmp_type);
        output.set_icmpv6_code(packet.icmp_code);

        // Write the payload
        output.set_payload(&payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(pnet_packet::icmpv6::checksum(
            &output.to_immutable(),
            &packet.source_address,
            &packet.destination_address,
        ));

        // Return the raw bytes
        output.packet().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use pnet_packet::icmpv6::Icmpv6Types;

    use super::*;

    // Test packet construction
    #[test]
    #[rustfmt::skip]
    fn test_packet_construction() {
        // Make a new packet
        let packet = Icmpv6Packet::new(
            "2001:db8:1::1".parse().unwrap(),
            "2001:db8:1::2".parse().unwrap(),
            Icmpv6Types::EchoRequest,
            Icmpv6Code(0),
            "Hello, world!".as_bytes().to_vec(),
        );

        // Convert to raw bytes
        let packet_bytes: Vec<u8> = packet.into();

        // Check the contents
        assert!(packet_bytes.len() >= 4 + 13);
        assert_eq!(packet_bytes[0], Icmpv6Types::EchoRequest.0);
        assert_eq!(packet_bytes[1], 0);
        assert_eq!(u16::from_be_bytes([packet_bytes[2], packet_bytes[3]]), 0xe2f0);
        assert_eq!(
            &packet_bytes[4..],
            "Hello, world!".as_bytes().to_vec().as_slice()
        );
    }
}
