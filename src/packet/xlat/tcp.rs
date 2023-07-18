use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::packet::{
    error::PacketError,
    protocols::{raw::RawBytes, tcp::TcpPacket},
};

/// Translates an IPv4 TCP packet to an IPv6 TCP packet
pub fn translate_tcp4_to_tcp6(
    input: TcpPacket<RawBytes>,
    new_source_addr: Ipv6Addr,
    new_destination_addr: Ipv6Addr,
) -> Result<TcpPacket<RawBytes>, PacketError> {
    // Build the packet
    Ok(TcpPacket::new(
        SocketAddr::new(IpAddr::V6(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V6(new_destination_addr), input.destination().port()),
        input.sequence,
        input.ack_number,
        input.flags,
        input.window_size,
        input.urgent_pointer,
        input.options,
        input.payload,
    )?)
}

/// Translates an IPv6 TCP packet to an IPv4 TCP packet
pub fn translate_tcp6_to_tcp4(
    input: TcpPacket<RawBytes>,
    new_source_addr: Ipv4Addr,
    new_destination_addr: Ipv4Addr,
) -> Result<TcpPacket<RawBytes>, PacketError> {
    // Build the packet
    Ok(TcpPacket::new(
        SocketAddr::new(IpAddr::V4(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V4(new_destination_addr), input.destination().port()),
        input.sequence,
        input.ack_number,
        input.flags,
        input.window_size,
        input.urgent_pointer,
        input.options,
        input.payload,
    )?)
}
