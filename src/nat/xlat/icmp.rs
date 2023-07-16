//! Translation logic for ICMP and ICMPv6

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pnet_packet::{
    icmp::{self, IcmpCode, IcmpPacket, IcmpType, MutableIcmpPacket},
    icmpv6::{self, Icmpv6Code, Icmpv6Packet, Icmpv6Type, MutableIcmpv6Packet},
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};

use crate::nat::packet::IpPacket;

fn remap_values_4to6(
    icmp_type: IcmpType,
    icmp_code: IcmpCode,
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
    payload: Vec<u8>,
) -> Option<(Icmpv6Type, Icmpv6Code, Vec<u8>)> {
    match icmp_type {
        // Destination Unreachable
        IcmpType(3) => match icmp_code {
            IcmpCode(0) => Some((Icmpv6Type(1), Icmpv6Code(0), payload)), // Destination network unreachable -> No route to destination
            IcmpCode(1) => Some((Icmpv6Type(1), Icmpv6Code(3), payload)), // Destination host unreachable -> Address unreachable
            IcmpCode(2) => Some((Icmpv6Type(1), Icmpv6Code(0), payload)), // Destination protocol unreachable -> No route to destination
            IcmpCode(3) => Some((Icmpv6Type(1), Icmpv6Code(4), payload)), // Destination port unreachable -> Port unreachable
            IcmpCode(4) => Some((Icmpv6Type(2), Icmpv6Code(0), vec![])), // Fragmentation required, and DF flag set -> Packet too big
            IcmpCode(5) => Some((Icmpv6Type(1), Icmpv6Code(5), payload)), // Source route failed -> Source address failed ingress/egress policy
            IcmpCode(6) => Some((Icmpv6Type(1), Icmpv6Code(0), payload)), // Destination network unknown -> No route to destination
            IcmpCode(7) => Some((Icmpv6Type(1), Icmpv6Code(3), payload)), // Destination host unknown -> Address unreachable
            IcmpCode(8) => Some((Icmpv6Type(1), Icmpv6Code(0), payload)), // Source host isolated -> No route to destination
            IcmpCode(9) => Some((Icmpv6Type(1), Icmpv6Code(1), payload)), // Network administratively prohibited -> Communication with destination administratively prohibited
            IcmpCode(10) => Some((Icmpv6Type(1), Icmpv6Code(1), payload)), // Host administratively prohibited -> Communication with destination administratively prohibited
            IcmpCode(11) => Some((Icmpv6Type(1), Icmpv6Code(0), payload)), // Network unreachable for ToS -> No route to destination
            IcmpCode(12) => Some((Icmpv6Type(1), Icmpv6Code(3), payload)), // Host unreachable for ToS -> Address unreachable
            IcmpCode(13) => Some((Icmpv6Type(1), Icmpv6Code(1), payload)), // Communication administratively prohibited -> Communication with destination administratively prohibited
            IcmpCode(14) => Some((Icmpv6Type(1), Icmpv6Code(1), payload)), // Host Precedence Violation -> Communication with destination administratively prohibited
            IcmpCode(15) => Some((Icmpv6Type(1), Icmpv6Code(1), payload)), // Precedence cutoff in effect -> Communication with destination administratively prohibited
            _ => Some((Icmpv6Type(1), Icmpv6Code(0), payload)),
        },

        // Time Exceeded
        IcmpType(11) => Some((Icmpv6Type(3), Icmpv6Code(icmp_code.0), {
            // The payload contains an IPv4 header and 8 bytes of data. This must also be translated
            let embedded_ipv4_packet = Ipv4Packet::new(&payload[4..]).unwrap();
            log::debug!("Embedded payload is: {:?}", embedded_ipv4_packet.payload());
            log::debug!(
                "Embedded next level protocol is: {}",
                embedded_ipv4_packet.get_next_level_protocol().0
            );

            // Build an IPv6 packet out of the IPv4 packet
            let mut embedded_ipv6_packet =
                MutableIpv6Packet::owned(vec![0u8; 40 + 4 + 8])
                    .unwrap();
            embedded_ipv6_packet.set_version(6);
            embedded_ipv6_packet.set_source(new_source);
            embedded_ipv6_packet.set_destination(new_destination);
            embedded_ipv6_packet.set_hop_limit(embedded_ipv4_packet.get_ttl());
            embedded_ipv6_packet.set_next_header(
                match embedded_ipv4_packet.get_next_level_protocol() {
                    IpNextHeaderProtocols::Icmp => IpNextHeaderProtocols::Icmpv6,
                    proto => proto,
                },
            );
            // embedded_ipv6_packet.set_payload_length(embedded_ipv4_packet.payload().len() as u16);
            embedded_ipv6_packet.set_payload_length(8u16);

            // Handle translating the embedded packet if it's ICMP
            match embedded_ipv4_packet.get_next_level_protocol() {
                IpNextHeaderProtocols::Icmp => {
                    let embedded_ipv4_packet_payload_bytes = embedded_ipv4_packet.payload();
                    let embedded_icmp_type = IcmpType(embedded_ipv4_packet_payload_bytes[0]);
                    let embedded_icmp_code = IcmpCode(embedded_ipv4_packet_payload_bytes[1]);
                    let embedded_icmp_payload = &embedded_ipv4_packet_payload_bytes[4..];

                    // Translate from ICMP to ICMPv6
                    let (embedded_icmpv6_type, embedded_icmpv6_code, embedded_icmpv6_payload) =
                        remap_values_4to6(
                            embedded_icmp_type,
                            embedded_icmp_code,
                            new_source,
                            new_destination,
                            embedded_icmp_payload.to_vec(),
                        )
                        .unwrap();

                    // Build an ICMPv6 packet out of the ICMPv6 values
                    let mut double_embedded_icmpv6_packet = MutableIcmpv6Packet::owned(vec![
                        0u8;
                        Icmpv6Packet::minimum_packet_size()
                            + embedded_icmpv6_payload.len()
                    ])
                    .unwrap();
                    double_embedded_icmpv6_packet.set_icmpv6_type(embedded_icmpv6_type);
                    double_embedded_icmpv6_packet.set_icmpv6_code(embedded_icmpv6_code);
                    double_embedded_icmpv6_packet.set_payload(&embedded_icmpv6_payload);
                    double_embedded_icmpv6_packet.set_checksum(0);
                    double_embedded_icmpv6_packet.set_checksum(icmpv6::checksum(
                        &double_embedded_icmpv6_packet.to_immutable(),
                        &new_source,
                        &new_destination,
                    ));

                    // Return the first 8 bytes of the embedded icmpv6 packet
                    embedded_ipv6_packet.set_payload(&double_embedded_icmpv6_packet.packet()[..8]);
                }
                _ => embedded_ipv6_packet.set_payload(embedded_ipv4_packet.payload()),
            };

            // Return the IPv6 packet
            embedded_ipv6_packet.packet().to_vec()
        })),

        // Echo Request
        IcmpType(8) => Some((Icmpv6Type(128), Icmpv6Code(0), payload)),

        // Echo Reply
        IcmpType(0) => Some((Icmpv6Type(129), Icmpv6Code(0), payload)),

        icmp_type => {
            log::warn!("ICMP type {} not supported", icmp_type.0);
            return None;
        }
    }
}

fn remap_values_6to4(
    icmp_type: Icmpv6Type,
    icmp_code: Icmpv6Code,
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
    payload: Vec<u8>,
) -> Option<(IcmpType, IcmpCode, Vec<u8>)> {
    match icmp_type {
        // Destination Unreachable
        Icmpv6Type(1) => match icmp_code {
            Icmpv6Code(0) => Some((IcmpType(3), IcmpCode(0), payload)), // No route to destination -> Destination network unreachable
            Icmpv6Code(3) => Some((IcmpType(3), IcmpCode(1), payload)), // Address unreachable -> Destination host unreachable
            Icmpv6Code(4) => Some((IcmpType(3), IcmpCode(3), payload)), // Port unreachable -> Destination port unreachable
            Icmpv6Code(5) => Some((IcmpType(3), IcmpCode(5), payload)), // Source route failed -> Source address failed ingress/egress policy
            Icmpv6Code(1) => Some((IcmpType(3), IcmpCode(13), payload)), // Communication administratively prohibited -> Communication administratively prohibited
            _ => Some((IcmpType(3), IcmpCode(0), payload)),
        },

        // Time Exceeded
        Icmpv6Type(3) => Some((IcmpType(11), IcmpCode(icmp_code.0), {
            // The payload contains an IPv6 header and 8 bytes of data. This must also be translated
            let embedded_ipv6_packet = Ipv6Packet::new(&payload).unwrap();
            log::debug!("Embedded payload is: {:?}", embedded_ipv6_packet.payload());
            log::debug!(
                "Embedded next header is: {}",
                embedded_ipv6_packet.get_next_header().0
            );

            // Build an IPv4 packet out of the IPv6 packet
            let mut embedded_ipv4_packet =
                MutableIpv4Packet::owned(vec![0u8; 20 + embedded_ipv6_packet.payload().len()])
                    .unwrap();
            embedded_ipv4_packet.set_version(4);
            embedded_ipv4_packet.set_source(new_source);
            embedded_ipv4_packet.set_destination(new_destination);
            embedded_ipv4_packet.set_ttl(embedded_ipv6_packet.get_hop_limit());
            embedded_ipv4_packet.set_next_level_protocol(
                match embedded_ipv6_packet.get_next_header() {
                    IpNextHeaderProtocols::Icmpv6 => IpNextHeaderProtocols::Icmp,
                    proto => proto,
                },
            );
            embedded_ipv4_packet.set_header_length(5);
            embedded_ipv4_packet.set_total_length(20 + embedded_ipv6_packet.payload().len() as u16);
            embedded_ipv4_packet.set_payload(embedded_ipv6_packet.payload());
            embedded_ipv4_packet.set_checksum(0);
            embedded_ipv4_packet.set_checksum(ipv4::checksum(&embedded_ipv4_packet.to_immutable()));

            // Return the IPv4 packet
            embedded_ipv4_packet.packet().to_vec()
        })),

        // Echo Request
        Icmpv6Type(128) => Some((IcmpType(8), IcmpCode(0), payload)),

        // Echo Reply
        Icmpv6Type(129) => Some((IcmpType(0), IcmpCode(0), payload)),

        icmp_type => {
            log::warn!("ICMPv6 type {} not supported", icmp_type.0);
            return None;
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IcmpProxyError {
    #[error("Packet too short. Got {0} bytes")]
    PacketTooShort(usize),
}

pub fn proxy_icmp_packet<'a>(
    original_packet: IpPacket<'a>,
    new_source: IpAddr,
    new_destination: IpAddr,
) -> Result<Option<IpPacket>, IcmpProxyError> {
    // Parse the original packet's payload to extract ICMP data
    let icmp_packet = original_packet.get_payload().to_vec();

    // Construct a new output packet
    match (original_packet, new_source, new_destination) {
        // Translate IPv4(ICMP) to IPv6(ICMPv6)
        (IpPacket::V4(original_packet), IpAddr::V6(new_source), IpAddr::V6(new_destination)) => {
            // Parse the ICMP packet
            let icmp_packet = IcmpPacket::new(&icmp_packet)
                .ok_or_else(|| IcmpProxyError::PacketTooShort(icmp_packet.len()))?;
            log::debug!(
                "Incoming packet has ICMP type: {}",
                icmp_packet.get_icmp_type().0
            );
            log::debug!(
                "Incoming packet has ICMP code: {}",
                icmp_packet.get_icmp_code().0
            );

            // Remap ICMP values to ICMPv6 ones
            if let Some((icmpv6_type, icmpv6_code, icmpv6_payload)) = remap_values_4to6(
                icmp_packet.get_icmp_type(),
                icmp_packet.get_icmp_code(),
                new_source,
                new_destination,
                icmp_packet.payload().to_vec(),
            ) {
                // Build an actual ICMPv6 packet out of the values
                let mut icmpv6_packet = MutableIcmpv6Packet::owned(vec![
                    0u8;
                    Icmpv6Packet::minimum_packet_size()
                        + icmpv6_payload.len()
                ])
                .unwrap();
                icmpv6_packet.set_icmpv6_type(icmpv6_type);
                icmpv6_packet.set_icmpv6_code(icmpv6_code);
                icmpv6_packet.set_payload(&icmpv6_payload);
                icmpv6_packet.set_checksum(0);
                icmpv6_packet.set_checksum(icmpv6::checksum(
                    &icmpv6_packet.to_immutable(),
                    &new_source,
                    &new_destination,
                ));

                // Build an IPv6 packet out of the ICMPv6 packet
                let mut output =
                    MutableIpv6Packet::owned(vec![0u8; 40 + icmpv6_packet.packet().len()]).unwrap();
                output.set_version(6);
                output.set_source(new_source);
                output.set_destination(new_destination);
                output.set_hop_limit(original_packet.get_ttl());
                output.set_next_header(IpNextHeaderProtocols::Icmpv6);
                output.set_payload_length(icmpv6_packet.packet().len() as u16);
                output.set_payload(icmpv6_packet.packet());

                // Return the IPv6 packet
                return Ok(Some(IpPacket::V6(
                    Ipv6Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
                )));
            }
            return Ok(None);
        }

        // Translate IPv6(ICMPv6) to IPv4(ICMP)
        (IpPacket::V6(original_packet), IpAddr::V4(new_source), IpAddr::V4(new_destination)) => {
            // Parse the ICMP packet
            let icmp_packet = Icmpv6Packet::new(&icmp_packet)
                .ok_or_else(|| IcmpProxyError::PacketTooShort(icmp_packet.len()))?;
            log::debug!(
                "Incoming packet has ICMPv6 type: {}",
                icmp_packet.get_icmpv6_type().0
            );
            log::debug!(
                "Incoming packet has ICMPv6 code: {}",
                icmp_packet.get_icmpv6_code().0
            );

            // Remap ICMPv6 values to ICMP ones
            if let Some((icmp_type, icmp_code, icmp_payload)) = remap_values_6to4(
                icmp_packet.get_icmpv6_type(),
                icmp_packet.get_icmpv6_code(),
                new_source,
                new_destination,
                icmp_packet.payload().to_vec(),
            ) {
                // Build an actual ICMP packet out of the values
                let mut icmp_packet = MutableIcmpPacket::owned(vec![
                    0u8;
                    IcmpPacket::minimum_packet_size(
                    ) + icmp_payload.len()
                ])
                .unwrap();
                icmp_packet.set_icmp_type(icmp_type);
                icmp_packet.set_icmp_code(icmp_code);
                icmp_packet.set_payload(&icmp_payload);
                icmp_packet.set_checksum(0);
                icmp_packet.set_checksum(icmp::checksum(&icmp_packet.to_immutable()));

                // Build an IPv4 packet out of the ICMP packet
                let mut output =
                    MutableIpv4Packet::owned(vec![0u8; 20 + icmp_packet.packet().len()]).unwrap();
                output.set_version(4);
                output.set_source(new_source);
                output.set_destination(new_destination);
                output.set_ttl(original_packet.get_hop_limit());
                output.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
                output.set_header_length(5);
                output.set_total_length(20 + icmp_packet.packet().len() as u16);
                output.set_payload(icmp_packet.packet());
                output.set_checksum(0);
                output.set_checksum(ipv4::checksum(&output.to_immutable()));

                // Return the IPv4 packet
                return Ok(Some(IpPacket::V4(
                    Ipv4Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
                )));
            }
            return Ok(None);
        }

        _ => unreachable!(),
    }
}
