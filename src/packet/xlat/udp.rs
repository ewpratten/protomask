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
    Ok(UdpPacket::new(
        SocketAddr::new(IpAddr::V6(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V6(new_destination_addr), input.destination().port()),
        input.payload,
    )?)
}

/// Translates an IPv6 UDP packet to an IPv4 UDP packet
pub fn translate_udp6_to_udp4(
    input: UdpPacket<RawBytes>,
    new_source_addr: Ipv4Addr,
    new_destination_addr: Ipv4Addr,
) -> Result<UdpPacket<RawBytes>, PacketError> {
    // Build the packet
    Ok(UdpPacket::new(
        SocketAddr::new(IpAddr::V4(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V4(new_destination_addr), input.destination().port()),
        input.payload,
    )?)
}
