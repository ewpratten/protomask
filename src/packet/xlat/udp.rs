use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::packet::{
    error::PacketError,
    protocols::{raw::RawBytes, udp::UdpPacket},
};

/// Translates an IPv4 UDP packet to an IPv6 UDP packet
pub fn translate_udp4_to_udp6(
    input: UdpPacket<RawBytes>,
    new_source_addr: Ipv6Addr,
    new_destination_addr: Ipv6Addr,
) -> Result<UdpPacket<RawBytes>, PacketError> {
    // Build the packet
    UdpPacket::new(
        SocketAddr::new(IpAddr::V6(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V6(new_destination_addr), input.destination().port()),
        input.payload,
    )
}

/// Translates an IPv6 UDP packet to an IPv4 UDP packet
pub fn translate_udp6_to_udp4(
    input: UdpPacket<RawBytes>,
    new_source_addr: Ipv4Addr,
    new_destination_addr: Ipv4Addr,
) -> Result<UdpPacket<RawBytes>, PacketError> {
    // Build the packet
    UdpPacket::new(
        SocketAddr::new(IpAddr::V4(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V4(new_destination_addr), input.destination().port()),
        input.payload,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::protocols::udp::UdpPacket;

    #[test]
    fn test_translate_udp4_to_udp6() {
        // Create an IPv4 UDP packet
        let ipv4_packet = UdpPacket::new(
            "192.0.2.1:1234".parse().unwrap(),
            "192.0.2.2:5678".parse().unwrap(),
            RawBytes("Hello, world!".as_bytes().to_vec()),
        )
        .unwrap();

        // Translate the packet to IPv6
        let ipv6_packet = translate_udp4_to_udp6(
            ipv4_packet,
            "2001:db8::1".parse().unwrap(),
            "2001:db8::2".parse().unwrap(),
        )
        .unwrap();

        // Ensure the translation is correct
        assert_eq!(ipv6_packet.source(), "[2001:db8::1]:1234".parse().unwrap());
        assert_eq!(
            ipv6_packet.destination(),
            "[2001:db8::2]:5678".parse().unwrap()
        );
        assert_eq!(
            ipv6_packet.payload,
            RawBytes("Hello, world!".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_translate_udp6_to_udp4() {
        // Create an IPv6 UDP packet
        let ipv6_packet = UdpPacket::new(
            "[2001:db8::1]:1234".parse().unwrap(),
            "[2001:db8::2]:5678".parse().unwrap(),
            RawBytes("Hello, world!".as_bytes().to_vec()),
        )
        .unwrap();

        // Translate the packet to IPv4
        let ipv4_packet = translate_udp6_to_udp4(
            ipv6_packet,
            "192.0.2.1".parse().unwrap(),
            "192.0.2.2".parse().unwrap(),
        )
        .unwrap();

        // Ensure the translation is correct
        assert_eq!(ipv4_packet.source(), "192.0.2.1:1234".parse().unwrap());
        assert_eq!(ipv4_packet.destination(), "192.0.2.2:5678".parse().unwrap());
        assert_eq!(
            ipv4_packet.payload,
            RawBytes("Hello, world!".as_bytes().to_vec())
        );
    }
}
