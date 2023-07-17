use std::net::{Ipv4Addr, Ipv6Addr};

use pnet_packet::{
    // ip::IpNextHeaderProtocols,
    // ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    // ipv6::{Ipv6Packet, MutableIpv6Packet},
    udp::{self, MutableUdpPacket, UdpPacket},
    Packet,
};

// use crate::nat::packet::IpPacket;

use super::PacketTranslationError;

// #[derive(Debug, thiserror::Error)]
// pub enum UdpProxyError {
//     #[error("Packet too short. Got {0} bytes")]
//     PacketTooShort(usize),
// }

// /// Extracts information from an original packet, and proxies UDP contents via a new source and destination
// pub async fn proxy_udp_packet<'a>(
//     original_packet: IpPacket<'a>,
//     new_source: IpAddr,
//     new_destination: IpAddr,
// ) -> Result<IpPacket, UdpProxyError> {
//     // Parse the original packet's payload to extract UDP data
//     let udp_packet = UdpPacket::new(original_packet.get_payload())
//         .ok_or_else(|| UdpProxyError::PacketTooShort(original_packet.get_payload().len()))?;
//     log::debug!(
//         "Incoming UDP packet ports: {} -> {}",
//         udp_packet.get_source(),
//         udp_packet.get_destination()
//     );
//     log::debug!(
//         "Incoming UDP packet payload len: {}",
//         udp_packet.payload().len()
//     );

//     // Construct a new output packet
//     match (&original_packet, new_source, new_destination) {
//         // Translate IPv4(UDP) to IPv6(UDP)
//         (IpPacket::V4(_), IpAddr::V6(new_source), IpAddr::V6(new_destination)) => {
//             // Construct translated UDP packet
//             let mut translated_udp_packet =
//                 MutableUdpPacket::owned(vec![0u8; 8 + udp_packet.payload().len()]).unwrap();
//             translated_udp_packet.set_source(udp_packet.get_source());
//             translated_udp_packet.set_destination(udp_packet.get_destination());
//             translated_udp_packet.set_length(8 + udp_packet.payload().len() as u16);
//             translated_udp_packet.set_payload(udp_packet.payload());
//             translated_udp_packet.set_checksum(0);
//             translated_udp_packet.set_checksum(udp::ipv6_checksum(
//                 &translated_udp_packet.to_immutable(),
//                 &new_source,
//                 &new_destination,
//             ));

//             // Construct translated IP packet to wrap UDP packet
//             let mut output =
//                 MutableIpv6Packet::owned(vec![0u8; 40 + translated_udp_packet.packet().len()])
//                     .unwrap();
//             output.set_version(6);
//             output.set_source(new_source);
//             output.set_destination(new_destination);
//             output.set_hop_limit(original_packet.get_ttl());
//             output.set_next_header(IpNextHeaderProtocols::Udp);
//             output.set_payload_length(translated_udp_packet.packet().len() as u16);
//             output.set_payload(translated_udp_packet.packet());
//             Ok(IpPacket::V6(
//                 Ipv6Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
//             ))
//         }

//         // Translate IPv6(UDP) to IPv4(UDP)
//         (IpPacket::V6(_), IpAddr::V4(new_source), IpAddr::V4(new_destination)) => {
//             // Construct translated UDP packet
//             let mut translated_udp_packet =
//                 MutableUdpPacket::owned(vec![0u8; 8 + udp_packet.payload().len()]).unwrap();
//             translated_udp_packet.set_source(udp_packet.get_source());
//             translated_udp_packet.set_destination(udp_packet.get_destination());
//             translated_udp_packet.set_length(8 + udp_packet.payload().len() as u16);
//             translated_udp_packet.set_payload(udp_packet.payload());
//             translated_udp_packet.set_checksum(0);
//             translated_udp_packet.set_checksum(udp::ipv4_checksum(
//                 &translated_udp_packet.to_immutable(),
//                 &new_source,
//                 &new_destination,
//             ));

//             // Construct translated IP packet to wrap UDP packet
//             let mut output =
//                 MutableIpv4Packet::owned(vec![0u8; 20 + translated_udp_packet.packet().len()])
//                     .unwrap();
//             output.set_version(4);
//             output.set_source(new_source);
//             output.set_destination(new_destination);
//             output.set_ttl(original_packet.get_ttl());
//             output.set_next_level_protocol(IpNextHeaderProtocols::Udp);
//             output.set_header_length(5);
//             output.set_total_length(20 + translated_udp_packet.packet().len() as u16);
//             output.set_payload(translated_udp_packet.packet());
//             output.set_checksum(0);
//             output.set_checksum(ipv4::checksum(&output.to_immutable()));
//             Ok(IpPacket::V4(
//                 Ipv4Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
//             ))
//         }

//         _ => unreachable!(),
//     }
// }

pub fn translate_udp_4_to_6(
    ipv4_udp: UdpPacket,
    new_source: Ipv6Addr,
    new_dest: Ipv6Addr,
) -> Result<UdpPacket, PacketTranslationError> {
    // Create a mutable clone of the IPv4 UDP packet, so it can be adapted for use in IPv6
    let mut ipv6_udp = MutableUdpPacket::owned(ipv4_udp.packet().to_vec())
        .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv4_udp.packet().len()))?;

    // Rewrite the checksum for use in an IPv6 packet
    ipv6_udp.set_checksum(0);
    ipv6_udp.set_checksum(udp::ipv6_checksum(
        &ipv4_udp.to_immutable(),
        &new_source,
        &new_dest,
    ));

    // Return the translated packet
    Ok(UdpPacket::owned(ipv6_udp.packet().to_vec()).unwrap())
}

pub fn translate_udp_6_to_4(
    ipv6_udp: UdpPacket,
    new_source: Ipv4Addr,
    new_dest: Ipv4Addr,
) -> Result<UdpPacket, PacketTranslationError> {
    // Create a mutable clone of the IPv6 UDP packet, so it can be adapted for use in IPv4
    let mut ipv4_udp = MutableUdpPacket::owned(ipv6_udp.packet().to_vec())
        .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv6_udp.packet().len()))?;

    // Rewrite the checksum for use in an IPv4 packet
    ipv4_udp.set_checksum(0);
    ipv4_udp.set_checksum(udp::ipv4_checksum(
        &ipv6_udp.to_immutable(),
        &new_source,
        &new_dest,
    ));

    // Return the translated packet
    Ok(UdpPacket::owned(ipv4_udp.packet().to_vec()).unwrap())
}
