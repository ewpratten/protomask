use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pnet_packet::ip::IpNextHeaderProtocols;

use crate::{
    packet::protocols::{icmp::IcmpPacket, tcp::TcpPacket, udp::UdpPacket},
    packet::{
        error::PacketError,
        protocols::{icmpv6::Icmpv6Packet, ipv4::Ipv4Packet, ipv6::Ipv6Packet, raw::RawBytes},
    },
};

use super::{
    icmp::{translate_icmp_to_icmpv6, translate_icmpv6_to_icmp},
    tcp::{translate_tcp4_to_tcp6, translate_tcp6_to_tcp4},
    udp::{translate_udp4_to_udp6, translate_udp6_to_udp4},
};

/// Translates an IPv4 packet to an IPv6 packet
pub fn translate_ipv4_to_ipv6(
    input: Ipv4Packet<Vec<u8>>,
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Ipv6Packet<Vec<u8>>, PacketError> {
    // Perform recursive translation to determine the new payload
    let new_payload = match input.protocol {
        IpNextHeaderProtocols::Icmp => {
            let icmp_input: IcmpPacket<RawBytes> = input.payload.try_into()?;
            translate_icmp_to_icmpv6(icmp_input, new_source, new_destination)?.into()
        }
        IpNextHeaderProtocols::Udp => {
            let udp_input: UdpPacket<RawBytes> = UdpPacket::new_from_bytes_raw_payload(
                &input.payload,
                IpAddr::V4(input.source_address),
                IpAddr::V4(input.destination_address),
            )?;
            translate_udp4_to_udp6(udp_input, new_source, new_destination)?.into()
        }
        IpNextHeaderProtocols::Tcp => {
            let tcp_input: TcpPacket<RawBytes> = TcpPacket::new_from_bytes_raw_payload(
                &input.payload,
                IpAddr::V4(input.source_address),
                IpAddr::V4(input.destination_address),
            )?;
            translate_tcp4_to_tcp6(tcp_input, new_source, new_destination)?.into()
        }
        _ => {
            log::warn!("Unsupported next level protocol: {}", input.protocol);
            input.payload
        }
    };

    // Build the output IPv6 packet
    let output = Ipv6Packet::new(
        0,
        0,
        match input.protocol {
            IpNextHeaderProtocols::Icmp => IpNextHeaderProtocols::Icmpv6,
            proto => proto,
        },
        input.ttl,
        new_source,
        new_destination,
        new_payload,
    );

    // Return the output
    Ok(output)
}

/// Translates an IPv6 packet to an IPv4 packet
pub fn translate_ipv6_to_ipv4(
    input: &Ipv6Packet<Vec<u8>>,
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<Ipv4Packet<Vec<u8>>, PacketError> {
    // Perform recursive translation to determine the new payload
    let new_payload = match input.next_header {
        IpNextHeaderProtocols::Icmpv6 => {
            let icmpv6_input: Icmpv6Packet<RawBytes> = Icmpv6Packet::new_from_bytes_raw_payload(
                &input.payload,
                input.source_address,
                input.destination_address,
            )?;
            Some(translate_icmpv6_to_icmp(icmpv6_input, new_source, new_destination)?.into())
        }
        IpNextHeaderProtocols::Udp => {
            let udp_input: UdpPacket<RawBytes> = UdpPacket::new_from_bytes_raw_payload(
                &input.payload,
                IpAddr::V6(input.source_address),
                IpAddr::V6(input.destination_address),
            )?;
            Some(translate_udp6_to_udp4(udp_input, new_source, new_destination)?.into())
        }
        IpNextHeaderProtocols::Tcp => {
            let tcp_input: TcpPacket<RawBytes> = TcpPacket::new_from_bytes_raw_payload(
                &input.payload,
                IpAddr::V6(input.source_address),
                IpAddr::V6(input.destination_address),
            )?;
            Some(translate_tcp6_to_tcp4(tcp_input, new_source, new_destination)?.into())
        }
        _ => {
            log::warn!("Unsupported next level protocol: {}", input.next_header);
            None
        }
    };

    // Build the output IPv4 packet
    let output = Ipv4Packet::new(
        0,
        0,
        0,
        0,
        0,
        input.hop_limit,
        match input.next_header {
            IpNextHeaderProtocols::Icmpv6 => IpNextHeaderProtocols::Icmp,
            proto => proto,
        },
        new_source,
        new_destination,
        vec![],
        new_payload.unwrap_or_default(),
    );

    // Return the output
    Ok(output)
}
