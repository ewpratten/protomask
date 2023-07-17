use std::net::{Ipv4Addr, Ipv6Addr};

use pnet_packet::{
    tcp::{self, MutableTcpPacket, TcpPacket},
    Packet,
};

use super::PacketTranslationError;

/// Translate an IPv4 TCP packet into an IPv6 TCP packet (aka: recalculate checksum)
pub fn translate_tcp_4_to_6(
    ipv4_tcp: TcpPacket,
    new_source: Ipv6Addr,
    new_dest: Ipv6Addr,
) -> Result<TcpPacket, PacketTranslationError> {
    // Create a mutable clone of the IPv4 TCP packet, so it can be adapted for use in IPv6
    let mut ipv6_tcp = MutableTcpPacket::owned(ipv4_tcp.packet().to_vec())
        .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv4_tcp.packet().len()))?;

    // Rewrite the checksum for use in an IPv6 packet
    ipv6_tcp.set_checksum(0);
    ipv6_tcp.set_checksum(tcp::ipv6_checksum(
        &ipv4_tcp.to_immutable(),
        &new_source,
        &new_dest,
    ));

    // Return the translated packet
    Ok(TcpPacket::owned(ipv6_tcp.packet().to_vec()).unwrap())
}

/// Translate an IPv6 TCP packet into an IPv4 TCP packet (aka: recalculate checksum)
pub fn translate_tcp_6_to_4(
    ipv6_tcp: TcpPacket,
    new_source: Ipv4Addr,
    new_dest: Ipv4Addr,
) -> Result<TcpPacket, PacketTranslationError> {
    // Create a mutable clone of the IPv6 TCP packet, so it can be adapted for use in IPv4
    let mut ipv4_tcp = MutableTcpPacket::owned(ipv6_tcp.packet().to_vec())
        .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv6_tcp.packet().len()))?;

    // Rewrite the checksum for use in an IPv4 packet
    ipv4_tcp.set_checksum(0);
    ipv4_tcp.set_checksum(tcp::ipv4_checksum(
        &ipv6_tcp.to_immutable(),
        &new_source,
        &new_dest,
    ));

    // Return the translated packet
    Ok(TcpPacket::owned(ipv4_tcp.packet().to_vec()).unwrap())
}
