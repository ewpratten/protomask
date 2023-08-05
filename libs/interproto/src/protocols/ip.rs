//! Translation functions that can convert packets between IPv4 and IPv6.

use super::{
    icmp::{translate_icmp_to_icmpv6, translate_icmpv6_to_icmp},
    tcp::{recalculate_tcp_checksum_ipv4, recalculate_tcp_checksum_ipv6},
    udp::{recalculate_udp_checksum_ipv4, recalculate_udp_checksum_ipv6},
};
use crate::error::{Error, Result};
use pnet::packet::{
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6Packet, MutableIpv6Packet},
    Packet,
};
use std::net::{Ipv4Addr, Ipv6Addr};

/// Translates an IPv4 packet into an IPv6 packet. The packet payload will be translated recursively as needed.
#[profiling::function]
pub fn translate_ipv4_to_ipv6(
    ipv4_packet: &[u8],
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Access the IPv4 packet data in a safe way
        let ipv4_packet = Ipv4Packet::new(ipv4_packet).ok_or(Error::PacketTooShort {
            expected: Ipv4Packet::minimum_packet_size(),
            actual: ipv4_packet.len(),
        })?;

        // Perform recursive translation to determine the new payload
        let new_payload = match ipv4_packet.get_next_level_protocol() {
            // Pass ICMP packets to the icmp-to-icmpv6 translator
            IpNextHeaderProtocols::Icmp => {
                translate_icmp_to_icmpv6(ipv4_packet.payload(), new_source, new_destination)?
            }

            // Pass TCP packets to the tcp translator
            IpNextHeaderProtocols::Tcp => {
                recalculate_tcp_checksum_ipv6(ipv4_packet.payload(), new_source, new_destination)?
            }

            // Pass UDP packets to the udp translator
            IpNextHeaderProtocols::Udp => {
                recalculate_udp_checksum_ipv6(ipv4_packet.payload(), new_source, new_destination)?
            }

            // If the next level protocol is not something we know how to translate,
            // just assume the payload can be passed through as-is
            protocol => {
                log::warn!("Unsupported next level protocol: {:?}", protocol);
                ipv4_packet.payload().to_vec()
            }
        };

        // Build a buffer to store the new IPv6 packet
        let mut output_buffer = vec![0u8; Ipv6Packet::minimum_packet_size() + new_payload.len()];

        // NOTE: There is no way this can fail since we are creating the buffer with explicitly enough space.
        let mut ipv6_packet =
            unsafe { MutableIpv6Packet::new(&mut output_buffer).unwrap_unchecked() };

        // Set the header fields
        ipv6_packet.set_version(6);
        ipv6_packet.set_next_header(match ipv4_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Icmp => IpNextHeaderProtocols::Icmpv6,
            proto => proto,
        });
        ipv6_packet.set_hop_limit(ipv4_packet.get_ttl());
        ipv6_packet.set_source(new_source);
        ipv6_packet.set_destination(new_destination);
        ipv6_packet.set_payload_length(new_payload.len().try_into().unwrap());

        // Copy the payload to the buffer
        ipv6_packet.set_payload(&new_payload);

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_IPV4, STATUS_TRANSLATED).inc();

        // Return the buffer
        Ok(output_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_IPV4, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

/// Translates an IPv6 packet into an IPv4 packet. The packet payload will be translated recursively as needed.
#[profiling::function]
pub fn translate_ipv6_to_ipv4(
    ipv6_packet: &[u8],
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Access the IPv6 packet data in a safe way
        let ipv6_packet = Ipv6Packet::new(ipv6_packet).ok_or(Error::PacketTooShort {
            expected: Ipv6Packet::minimum_packet_size(),
            actual: ipv6_packet.len(),
        })?;

        // Perform recursive translation to determine the new payload
        let new_payload = match ipv6_packet.get_next_header() {
            // Pass ICMP packets to the icmpv6-to-icmp translator
            IpNextHeaderProtocols::Icmpv6 => {
                translate_icmpv6_to_icmp(ipv6_packet.payload(), new_source, new_destination)?
            }

            // Pass TCP packets to the tcp translator
            IpNextHeaderProtocols::Tcp => {
                recalculate_tcp_checksum_ipv4(ipv6_packet.payload(), new_source, new_destination)?
            }

            // Pass UDP packets to the udp translator
            IpNextHeaderProtocols::Udp => {
                recalculate_udp_checksum_ipv4(ipv6_packet.payload(), new_source, new_destination)?
            }

            // If the next header is not something we know how to translate,
            // just assume the payload can be passed through as-is
            protocol => {
                log::warn!("Unsupported next header: {:?}", protocol);
                ipv6_packet.payload().to_vec()
            }
        };

        // Build a buffer to store the new IPv4 packet
        let mut output_buffer = vec![0u8; Ipv4Packet::minimum_packet_size() + new_payload.len()];

        // NOTE: There is no way this can fail since we are creating the buffer with explicitly enough space.
        let mut ipv4_packet =
            unsafe { MutableIpv4Packet::new(&mut output_buffer).unwrap_unchecked() };

        // Set the header fields
        ipv4_packet.set_version(4);
        ipv4_packet.set_header_length(5);
        ipv4_packet.set_ttl(ipv6_packet.get_hop_limit());
        ipv4_packet.set_next_level_protocol(match ipv6_packet.get_next_header() {
            IpNextHeaderProtocols::Icmpv6 => IpNextHeaderProtocols::Icmp,
            proto => proto,
        });
        ipv4_packet.set_source(new_source);
        ipv4_packet.set_destination(new_destination);
        ipv4_packet.set_total_length(
            (Ipv4Packet::minimum_packet_size() + new_payload.len())
                .try_into()
                .unwrap(),
        );

        // Copy the payload to the buffer
        ipv4_packet.set_payload(&new_payload);

        // Calculate the checksum
        ipv4_packet.set_checksum(ipv4::checksum(&ipv4_packet.to_immutable()));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_IPV6, STATUS_TRANSLATED).inc();

        // Return the buffer
        Ok(output_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_IPV6, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}
