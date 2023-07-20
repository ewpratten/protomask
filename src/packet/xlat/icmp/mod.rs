use std::net::{Ipv4Addr, Ipv6Addr};

use pnet_packet::{icmp::IcmpTypes, icmpv6::Icmpv6Types};

use crate::{
    metrics::ICMP_COUNTER,
    packet::{
        error::PacketError,
        protocols::{icmp::IcmpPacket, icmpv6::Icmpv6Packet, raw::RawBytes},
    },
};

use super::ip::{translate_ipv4_to_ipv6, translate_ipv6_to_ipv4};

mod type_code;

/// Translates an ICMP packet to an ICMPv6 packet
pub fn translate_icmp_to_icmpv6(
    input: IcmpPacket<RawBytes>,
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Icmpv6Packet<RawBytes>, PacketError> {
    ICMP_COUNTER
        .with_label_values(&[
            "icmp",
            &input.icmp_type.0.to_string(),
            &input.icmp_code.0.to_string(),
        ])
        .inc();

    // Translate the type and code
    let (icmpv6_type, icmpv6_code) =
        type_code::translate_type_and_code_4_to_6(input.icmp_type, input.icmp_code)?;

    // Some ICMP types require special payload edits
    let payload = match icmpv6_type {
        Icmpv6Types::TimeExceeded => {
            // In this case, the current payload looks like: 4bytes + Ipv4(Data)
            // This needs to be translated to: 4bytes + Ipv6(Data)
            let inner_payload = input.payload.0[4..].to_vec();

            // Translate
            let inner_payload =
                translate_ipv4_to_ipv6(inner_payload.try_into()?, new_source, new_destination)?;
            let inner_payload: Vec<u8> = inner_payload.into();

            // Build the new payload
            RawBytes({
                let mut buffer = Vec::with_capacity(4 + inner_payload.len());
                buffer.extend_from_slice(&input.payload.0[..4]);
                buffer.extend_from_slice(&inner_payload);
                buffer
            })
        }
        _ => input.payload,
    };

    // Build output packet
    Ok(Icmpv6Packet::new(
        new_source,
        new_destination,
        icmpv6_type,
        icmpv6_code,
        payload,
    ))
}

/// Translates an ICMPv6 packet to an ICMP packet
pub fn translate_icmpv6_to_icmp(
    input: Icmpv6Packet<RawBytes>,
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<IcmpPacket<RawBytes>, PacketError> {
    ICMP_COUNTER
        .with_label_values(&[
            "icmpv6",
            &input.icmp_type.0.to_string(),
            &input.icmp_code.0.to_string(),
        ])
        .inc();

    // Translate the type and code
    let (icmp_type, icmp_code) =
        type_code::translate_type_and_code_6_to_4(input.icmp_type, input.icmp_code)?;

    // Some ICMP types require special payload edits
    let payload = match icmp_type {
        IcmpTypes::TimeExceeded => {
            // In this case, the current payload looks like: 4bytes + Ipv6(Data)
            // This needs to be translated to: 4bytes + Ipv4(Data)
            let inner_payload = input.payload.0[4..].to_vec();

            // Translate
            let inner_payload =
                translate_ipv6_to_ipv4(inner_payload.try_into()?, new_source, new_destination)?;
            let inner_payload: Vec<u8> = inner_payload.into();

            // Build the new payload
            RawBytes({
                let mut buffer = Vec::with_capacity(4 + inner_payload.len());
                buffer.extend_from_slice(&input.payload.0[..4]);
                buffer.extend_from_slice(&inner_payload);
                buffer
            })
        }
        _ => input.payload,
    };

    // Build output packet
    Ok(IcmpPacket::new(icmp_type, icmp_code, payload))
}
