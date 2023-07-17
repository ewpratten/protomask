use std::net::IpAddr;

use pnet_packet::{
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    tcp::{self, MutableTcpPacket, TcpPacket},
    Packet,
};

use crate::nat::packet::IpPacket;

#[derive(Debug, thiserror::Error)]
pub enum TcpProxyError {
    #[error("Packet too short. Got {0} bytes")]
    PacketTooShort(usize),
}

/// Extracts information from an original packet, and proxies TCP contents via a new source and destination
pub async fn proxy_tcp_packet<'a>(
    original_packet: IpPacket<'a>,
    new_source: IpAddr,
    new_destination: IpAddr,
) -> Result<IpPacket, TcpProxyError> {
    // Parse the original packet's payload to extract UDP data
    let tcp_packet = TcpPacket::new(original_packet.get_payload())
        .ok_or_else(|| TcpProxyError::PacketTooShort(original_packet.get_payload().len()))?;
    log::debug!(
        "Incoming TCP packet ports: {} -> {}",
        tcp_packet.get_source(),
        tcp_packet.get_destination()
    );
    log::debug!(
        "Incoming TCP packet payload len: {}",
        tcp_packet.payload().len()
    );

    // Construct a new output packet
    match (&original_packet, new_source, new_destination) {
        // Translate IPv4(UDP) to IPv6(UDP)
        (IpPacket::V4(_), IpAddr::V6(new_source), IpAddr::V6(new_destination)) => {
            // Construct translated TCP packet
            let mut translated_tcp_packet =
                MutableTcpPacket::owned(tcp_packet.packet().to_vec()).unwrap();

            // Rewrite the checksum
            translated_tcp_packet.set_checksum(0);
            translated_tcp_packet.set_checksum(tcp::ipv6_checksum(
                &translated_tcp_packet.to_immutable(),
                &new_source,
                &new_destination,
            ));

            // Construct translated IP packet to wrap TCP packet
            let mut output =
                MutableIpv6Packet::owned(vec![0u8; 40 + translated_tcp_packet.packet().len()])
                    .unwrap();
            output.set_version(6);
            output.set_source(new_source);
            output.set_destination(new_destination);
            output.set_hop_limit(original_packet.get_ttl());
            output.set_next_header(IpNextHeaderProtocols::Tcp);
            output.set_payload_length(translated_tcp_packet.packet().len() as u16);
            output.set_payload(translated_tcp_packet.packet());
            Ok(IpPacket::V6(
                Ipv6Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
            ))
        }

        // Translate IPv6(UDP) to IPv4(UDP)
        (IpPacket::V6(_), IpAddr::V4(new_source), IpAddr::V4(new_destination)) => {
            // Construct translated TCP packet
            let mut translated_tcp_packet =
                MutableTcpPacket::owned(tcp_packet.packet().to_vec()).unwrap();

            // Rewrite the checksum
            translated_tcp_packet.set_checksum(0);
            translated_tcp_packet.set_checksum(tcp::ipv4_checksum(
                &translated_tcp_packet.to_immutable(),
                &new_source,
                &new_destination,
            ));

            // Construct translated IP packet to wrap TCP packet
            let mut output =
                MutableIpv4Packet::owned(vec![0u8; 20 + translated_tcp_packet.packet().len()]).unwrap();
            output.set_version(4);
            output.set_source(new_source);
            output.set_destination(new_destination);
            output.set_ttl(original_packet.get_ttl());
            output.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
            output.set_header_length(5);
            output.set_total_length(20 + translated_tcp_packet.packet().len() as u16);
            output.set_payload(translated_tcp_packet.packet());
            output.set_checksum(0);
            output.set_checksum(ipv4::checksum(&output.to_immutable()));
            Ok(IpPacket::V4(
                Ipv4Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
            ))
        }

        _ => unreachable!(),
    }
}
