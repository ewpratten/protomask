//! A generic internet protocol packet type

use std::net::IpAddr;

use pnet_packet::{ip::IpNextHeaderProtocol, ipv4::Ipv4Packet, ipv6::Ipv6Packet, Packet};

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("Packed too small (len: {0})")]
    PacketTooSmall(usize),
    #[error("Unknown Internet Protocol version: {0}")]
    UnknownVersion(u8),
}

/// A protocol-agnostic packet type
#[derive(Debug)]
pub enum IpPacket<'a> {
    /// IPv4 packet
    V4(Ipv4Packet<'a>),
    /// IPv6 packet
    V6(Ipv6Packet<'a>),
}

impl IpPacket<'_> {
    /// Creates a new packet from a byte slice
    pub fn new<'a>(bytes: &'a [u8]) -> Result<IpPacket<'a>, PacketError> {
        // Parse the packet. If there is an error, cast None to the error type
        match bytes[0] >> 4 {
            4 => Ok(IpPacket::V4(
                Ipv4Packet::new(bytes).ok_or_else(|| PacketError::PacketTooSmall(bytes.len()))?,
            )),
            6 => Ok(IpPacket::V6(
                Ipv6Packet::new(bytes).ok_or_else(|| PacketError::PacketTooSmall(bytes.len()))?,
            )),
            n => Err(PacketError::UnknownVersion(n)),
        }
    }

    /// Returns the source address
    pub fn get_source(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_source()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_source()),
        }
    }

    /// Returns the destination address
    pub fn get_destination(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_destination()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_destination()),
        }
    }

    /// Returns the packet header
    pub fn get_header(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.packet()[..20].as_ref(),
            IpPacket::V6(packet) => packet.packet()[..40].as_ref(),
        }
    }

    /// Returns the packet payload
    pub fn get_payload(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.payload(),
            IpPacket::V6(packet) => packet.payload(),
        }
    }

    /// Converts the packet to a byte vector
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IpPacket::V4(packet) => packet.packet().to_vec(),
            IpPacket::V6(packet) => packet.packet().to_vec(),
        }
    }

    /// Returns the packet length
    pub fn len(&self) -> usize {
        match self {
            IpPacket::V4(packet) => packet.packet().len(),
            IpPacket::V6(packet) => packet.packet().len(),
        }
    }

    /// Get the next header
    pub fn get_next_header(&self) -> IpNextHeaderProtocol {
        match self {
            IpPacket::V4(packet) => packet.get_next_level_protocol(),
            IpPacket::V6(packet) => packet.get_next_header(),
        }
    }

    /// Get the TTL
    pub fn get_ttl(&self) -> u8 {
        match self {
            IpPacket::V4(packet) => packet.get_ttl(),
            IpPacket::V6(packet) => packet.get_hop_limit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pnet_packet::{ipv4::MutableIpv4Packet, ipv6::MutableIpv6Packet};

    use super::*;

    #[test]
    fn test_ipv4_packet() {
        // Build packet to test
        let mut packet = MutableIpv4Packet::owned(vec![0; 20]).unwrap();
        packet.set_version(4);
        packet.set_source("192.0.2.1".parse().unwrap());
        packet.set_destination("192.0.2.2".parse().unwrap());

        // Parse
        let header = packet.packet()[..20].to_vec();
        let packet = IpPacket::new(packet.packet()).unwrap();
        assert_eq!(
            packet.get_source(),
            IpAddr::V4("192.0.2.1".parse().unwrap())
        );
        assert_eq!(
            packet.get_destination(),
            IpAddr::V4("192.0.2.2".parse().unwrap())
        );
        assert_eq!(packet.get_header(), header);
    }

    #[test]
    fn test_ipv6_packet() {
        // Build packet to test
        let mut packet = MutableIpv6Packet::owned(vec![0; 40]).unwrap();
        packet.set_version(6);
        packet.set_source("2001:db8::c0a8:1".parse().unwrap());
        packet.set_destination("2001:db8::c0a8:2".parse().unwrap());

        // Parse
        let header = packet.packet()[..40].to_vec();
        let packet = IpPacket::new(packet.packet()).unwrap();

        // Test
        assert_eq!(
            packet.get_source(),
            IpAddr::V6("2001:db8::c0a8:1".parse().unwrap())
        );
        assert_eq!(
            packet.get_destination(),
            IpAddr::V6("2001:db8::c0a8:2".parse().unwrap())
        );
        assert_eq!(packet.get_header(), header);
    }
}
