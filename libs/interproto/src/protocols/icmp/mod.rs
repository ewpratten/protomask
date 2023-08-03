use crate::{
    error::{Error, Result},
    protocols::ip::translate_ipv4_to_ipv6,
};
use pnet::packet::{
    icmp::{self, IcmpPacket, IcmpTypes, MutableIcmpPacket},
    icmpv6::{self, Icmpv6Packet, Icmpv6Types, MutableIcmpv6Packet},
    Packet,
};
use std::net::{Ipv4Addr, Ipv6Addr};

use super::ip::translate_ipv6_to_ipv4;

mod type_code;

/// Translate an ICMP packet to ICMPv6. This will make a best guess at the ICMPv6 type and code since there is no 1:1 mapping.
#[allow(clippy::deprecated_cfg_attr)]
pub fn translate_icmp_to_icmpv6(
    icmp_packet: &[u8],
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Access the ICMP packet data in a safe way
        let icmp_packet = IcmpPacket::new(icmp_packet).ok_or(Error::PacketTooShort {
            expected: IcmpPacket::minimum_packet_size(),
            actual: icmp_packet.len(),
        })?;

        // Track the incoming packet's type and code
        #[cfg(feature = "metrics")]
        protomask_metrics::metrics::ICMP_COUNTER
            .with_label_values(&[
                protomask_metrics::metrics::label_values::PROTOCOL_ICMP,
                &icmp_packet.get_icmp_type().0.to_string(),
                &icmp_packet.get_icmp_code().0.to_string(),
            ])
            .inc();

        // Translate the ICMP type and code to their ICMPv6 equivalents
        let (icmpv6_type, icmpv6_code) = type_code::translate_type_and_code_4_to_6(
            icmp_packet.get_icmp_type(),
            icmp_packet.get_icmp_code(),
        )?;

        // Some ICMP types require special payload edits
        let payload = match icmpv6_type {
            Icmpv6Types::TimeExceeded => {
                // Time exceeded messages contain the original IPv4 header and part of the payload. (with 4 bytes of forward padding)
                // We need to translate the IPv4 header and the payload, but keep the padding
                let mut output = vec![0u8; 4];
                output.copy_from_slice(&icmp_packet.payload()[..4]);
                output.extend_from_slice(&translate_ipv4_to_ipv6(
                    &icmp_packet.payload()[4..],
                    new_source,
                    new_destination,
                )?);
                output
            }
            _ => icmp_packet.payload().to_vec(),
        };

        // Build a buffer to store the new ICMPv6 packet
        let mut output_buffer = vec![0u8; IcmpPacket::minimum_packet_size() + payload.len()];

        // NOTE: There is no way this can fail since we are creating the buffer with explicitly enough space.
        let mut icmpv6_packet =
            unsafe { MutableIcmpv6Packet::new(&mut output_buffer).unwrap_unchecked() };

        // Set the header fields
        icmpv6_packet.set_icmpv6_type(icmpv6_type);
        icmpv6_packet.set_icmpv6_code(icmpv6_code);
        icmpv6_packet.set_checksum(0);

        // Copy the payload
        icmpv6_packet.set_payload(&payload);

        // Calculate the checksum
        icmpv6_packet.set_checksum(icmpv6::checksum(
            &icmpv6_packet.to_immutable(),
            &new_source,
            &new_destination,
        ));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_ICMP, STATUS_TRANSLATED).inc();

        // Return the translated packet
        Ok(output_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_ICMP, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

/// Translate an ICMPv6 packet to ICMP. This will make a best guess at the ICMP type and code since there is no 1:1 mapping.
pub fn translate_icmpv6_to_icmp(
    icmpv6_packet: &[u8],
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Access the ICMPv6 packet data in a safe way
        let icmpv6_packet = Icmpv6Packet::new(icmpv6_packet).ok_or(Error::PacketTooShort {
            expected: Icmpv6Packet::minimum_packet_size(),
            actual: icmpv6_packet.len(),
        })?;

        // Track the incoming packet's type and code
        #[cfg(feature = "metrics")]
        protomask_metrics::metrics::ICMP_COUNTER
            .with_label_values(&[
                protomask_metrics::metrics::label_values::PROTOCOL_ICMPV6,
                &icmpv6_packet.get_icmpv6_type().0.to_string(),
                &icmpv6_packet.get_icmpv6_code().0.to_string(),
            ])
            .inc();

        // Translate the ICMPv6 type and code to their ICMP equivalents
        let (icmp_type, icmp_code) = type_code::translate_type_and_code_6_to_4(
            icmpv6_packet.get_icmpv6_type(),
            icmpv6_packet.get_icmpv6_code(),
        )?;

        // Some ICMP types require special payload edits
        let payload = match icmp_type {
            IcmpTypes::TimeExceeded => {
                // Time exceeded messages contain the original IPv6 header and part of the payload. (with 4 bytes of forward padding)
                // We need to translate the IPv6 header and the payload, but keep the padding
                let mut output = vec![0u8; 4];
                output.copy_from_slice(&icmpv6_packet.payload()[..4]);
                output.extend_from_slice(&translate_ipv6_to_ipv4(
                    &icmpv6_packet.payload()[4..],
                    new_source,
                    new_destination,
                )?);
                output
            }
            _ => icmpv6_packet.payload().to_vec(),
        };

        // Build a buffer to store the new ICMP packet
        let mut output_buffer = vec![0u8; Icmpv6Packet::minimum_packet_size() + payload.len()];

        // NOTE: There is no way this can fail since we are creating the buffer with explicitly enough space.
        let mut icmp_packet =
            unsafe { MutableIcmpPacket::new(&mut output_buffer).unwrap_unchecked() };

        // Set the header fields
        icmp_packet.set_icmp_type(icmp_type);
        icmp_packet.set_icmp_code(icmp_code);

        // Copy the payload
        icmp_packet.set_payload(&payload);

        // Calculate the checksum
        icmp_packet.set_checksum(icmp::checksum(&icmp_packet.to_immutable()));

        // Return the translated packet
        Ok(output_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_ICMPV6, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}
