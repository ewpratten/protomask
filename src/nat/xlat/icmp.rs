use std::net::{Ipv4Addr, Ipv6Addr};

use pnet_packet::{
    icmp::{
        self, destination_unreachable, IcmpCode, IcmpPacket, IcmpType, IcmpTypes, MutableIcmpPacket,
    },
    icmpv6::{self, Icmpv6Code, Icmpv6Packet, Icmpv6Type, Icmpv6Types, MutableIcmpv6Packet},
    ip::IpNextHeaderProtocols,
    ipv4::Ipv4Packet,
    ipv6::Ipv6Packet,
    Packet,
};

use crate::{icmp_packet, icmpv6_packet, ipv4_packet, ipv6_packet};

use super::PacketTranslationError;

/// Best effort translation from an ICMP type and code to an ICMPv6 type and code
fn translate_type_and_code_4_to_6(
    icmp_type: IcmpType,
    icmp_code: IcmpCode,
) -> Option<(Icmpv6Type, Icmpv6Code)> {
    match (icmp_type, icmp_code) {
        // Echo Request
        (IcmpTypes::EchoRequest, _) => Some((Icmpv6Types::EchoRequest, Icmpv6Code(0))),

        // Echo Reply
        (IcmpTypes::EchoReply, _) => Some((Icmpv6Types::EchoReply, Icmpv6Code(0))),

        // Packet Too Big
        (
            IcmpTypes::DestinationUnreachable,
            destination_unreachable::IcmpCodes::FragmentationRequiredAndDFFlagSet,
        ) => Some((Icmpv6Types::PacketTooBig, Icmpv6Code(0))),

        // Destination Unreachable
        (IcmpTypes::DestinationUnreachable, icmp_code) => Some((
            Icmpv6Types::DestinationUnreachable,
            #[cfg_attr(rustfmt, rustfmt_skip)]
            Icmpv6Code(match icmp_code {
                destination_unreachable::IcmpCodes::DestinationHostUnreachable => 3,
                destination_unreachable::IcmpCodes::DestinationProtocolUnreachable => 4,
                destination_unreachable::IcmpCodes::DestinationPortUnreachable => 4,
                destination_unreachable::IcmpCodes::SourceRouteFailed => 5,
                destination_unreachable::IcmpCodes::SourceHostIsolated => 2,
                destination_unreachable::IcmpCodes::NetworkAdministrativelyProhibited => 1,
                destination_unreachable::IcmpCodes::HostAdministrativelyProhibited => 1,
                destination_unreachable::IcmpCodes::CommunicationAdministrativelyProhibited => 1,

                // Default to No Route to Destination
                _ => 0,
            }),
        )),

        // Time Exceeded
        (IcmpTypes::TimeExceeded, icmp_code) => {
            Some((Icmpv6Types::TimeExceeded, Icmpv6Code(icmp_code.0)))
        }

        // Default unsupported
        _ => {
            log::warn!(
                "Unsupported ICMP code and type: {:?}, {:?}",
                icmp_type,
                icmp_code
            );
            None
        }
    }
}

/// Best effort translation from an ICMPv6 type and code to an ICMP type and code
fn translate_type_and_code_6_to_4(
    icmp_type: Icmpv6Type,
    icmp_code: Icmpv6Code,
) -> Option<(IcmpType, IcmpCode)> {
    match (icmp_type, icmp_code) {
        // Echo Request
        (Icmpv6Types::EchoRequest, _) => Some((IcmpTypes::EchoRequest, IcmpCode(0))),

        // Echo Reply
        (Icmpv6Types::EchoReply, _) => Some((IcmpTypes::EchoReply, IcmpCode(0))),

        // Packet Too Big
        (Icmpv6Types::PacketTooBig, _) => Some((
            IcmpTypes::DestinationUnreachable,
            destination_unreachable::IcmpCodes::FragmentationRequiredAndDFFlagSet,
        )),

        // Destination Unreachable
        (Icmpv6Types::DestinationUnreachable, icmp_code) => Some((
            IcmpTypes::DestinationUnreachable,
            #[cfg_attr(rustfmt, rustfmt_skip)]
            match icmp_code.0 {
                1 => destination_unreachable::IcmpCodes::CommunicationAdministrativelyProhibited,
                2 => destination_unreachable::IcmpCodes::SourceHostIsolated,
                3 => destination_unreachable::IcmpCodes::DestinationHostUnreachable,
                4 => destination_unreachable::IcmpCodes::DestinationPortUnreachable,
                5 => destination_unreachable::IcmpCodes::SourceRouteFailed,
                _ => destination_unreachable::IcmpCodes::DestinationNetworkUnreachable,
            },
        )),

        // Time Exceeded
        (Icmpv6Types::TimeExceeded, icmp_code) => {
            Some((IcmpTypes::TimeExceeded, IcmpCode(icmp_code.0)))
        }

        // Default unsupported
        _ => {
            log::warn!(
                "Unsupported ICMPv6 code and type: {:?}, {:?}",
                icmp_type,
                icmp_code
            );
            None
        }
    }
}

/// Translate an ICMP packet into an ICMPv6 packet
pub fn translate_icmp_4_to_6(
    icmp_packet: IcmpPacket,
    new_source: Ipv6Addr,
    new_dest: Ipv6Addr,
) -> Result<Option<Icmpv6Packet>, PacketTranslationError> {
    // Translate the type and code
    if let Some((icmpv6_type, icmpv6_code)) =
        translate_type_and_code_4_to_6(icmp_packet.get_icmp_type(), icmp_packet.get_icmp_code())
    {
        // "Time Exceeded" requires an additional payload be embedded in the packet
        // This payload looks like: 4bytes + IPv6(data)
        let mut output_payload = icmp_packet.payload().to_vec();
        if icmpv6_type == Icmpv6Types::TimeExceeded {
            // Get access to the original payload
            let original_payload =
                Ipv4Packet::new(&icmp_packet.payload()[4..]).ok_or_else(|| {
                    PacketTranslationError::EmbeddedPacketTooShort(icmp_packet.payload().len() - 4)
                })?;

            // Copy the original payload's payload to a buffer
            let mut original_payload_inner = vec![0u8; original_payload.payload().len()];
            original_payload_inner.copy_from_slice(original_payload.payload());

            // if the original payload's next header is ICMP, we need to translated the inner payload's ICMP type
            if original_payload.get_next_level_protocol() == IpNextHeaderProtocols::Icmp {
                log::debug!("Time Exceeded packet contains another ICMP packet.. Translating");
                if let Some((icmpv6_type, icmpv6_code)) = translate_type_and_code_4_to_6(
                    IcmpType(original_payload_inner[0]),
                    IcmpCode(original_payload_inner[1]),
                ) {
                    let inner_icmpv6 = icmpv6_packet!(
                        new_source,
                        new_dest,
                        icmpv6_type,
                        icmpv6_code,
                        &original_payload_inner[4..]
                    );
                    original_payload_inner = inner_icmpv6.packet().to_vec();
                    log::debug!(
                        "Translated inner ICMPv6 packet: {:?}",
                        original_payload_inner
                    );
                }
            }

            // Build a new IPv6 packet out of the embedded IPv4 packet's data
            let new_payload_packet = ipv6_packet!(
                new_source,
                new_dest,
                match original_payload.get_next_level_protocol() {
                    IpNextHeaderProtocols::Icmp => IpNextHeaderProtocols::Icmpv6,
                    proto => proto,
                },
                original_payload.get_ttl(),
                &original_payload_inner
            );

            // Set the payload
            output_payload = vec![0u8; 4 + new_payload_packet.packet().len()];
            output_payload[4..].copy_from_slice(new_payload_packet.packet());
        }

        // Create a new ICMPv6 packet for the translated values to be stored in
        let mut output = MutableIcmpv6Packet::owned(vec![
            0u8;
            Icmpv6Packet::minimum_packet_size()
                + output_payload.len()
        ])
        .unwrap();

        // Set the type and code
        output.set_icmpv6_type(icmpv6_type);
        output.set_icmpv6_code(icmpv6_code);

        // Set the payload
        log::debug!("Setting ICMPv6 payload: {:?}", output_payload);
        output.set_payload(&output_payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(icmpv6::checksum(
            &output.to_immutable(),
            &new_source,
            &new_dest,
        ));

        // Return the translated packet
        return Ok(Some(
            Icmpv6Packet::owned(output.to_immutable().packet().to_vec()).unwrap(),
        ));
    }

    Ok(None)
}

/// Translate an ICMPv6 packet into an ICMP packet
pub fn translate_icmp_6_to_4(
    icmpv6_packet: Icmpv6Packet,
    new_source: Ipv4Addr,
    new_dest: Ipv4Addr,
) -> Result<Option<IcmpPacket>, PacketTranslationError> {
    // If the incoming packet is a "Parameter Problem", log it
    if icmpv6_packet.get_icmpv6_type() == Icmpv6Types::ParameterProblem {
        log::warn!(
            "ICMPv6 Parameter Problem: {:?}",
            match icmpv6_packet.get_icmpv6_code().0 {
                0 => "Erroneous header field encountered",
                1 => "Unrecognized Next Header type encountered",
                2 => "Unrecognized IPv6 option encountered",
                _ => "Unknown",
            }
        );
    }

    // Translate the type and code
    if let Some((icmp_type, icmp_code)) = translate_type_and_code_6_to_4(
        icmpv6_packet.get_icmpv6_type(),
        icmpv6_packet.get_icmpv6_code(),
    ) {
        // "Time Exceeded" requires an additional payload be embedded in the packet
        // This payload looks like: 4bytes + IPv6(8bytes)
        let mut output_payload = icmpv6_packet.payload().to_vec();
        if icmp_type == IcmpTypes::TimeExceeded {
            // Get access to the original payload
            let original_payload =
                Ipv6Packet::new(&icmpv6_packet.payload()[4..]).ok_or_else(|| {
                    PacketTranslationError::EmbeddedPacketTooShort(
                        icmpv6_packet.payload().len() - 4,
                    )
                })?;

            // Copy the original payload's payload to a buffer
            let mut original_payload_inner = vec![0u8; original_payload.payload().len()];
            original_payload_inner.copy_from_slice(original_payload.payload());

            // if the original payload's next header is ICMPv6, we need to translated the inner payload's ICMPv6 type
            if original_payload.get_next_header() == IpNextHeaderProtocols::Icmpv6 {
                log::debug!("Time Exceeded packet contains another ICMPv6 packet.. Translating");
                if let Some((icmp_type, icmp_code)) = translate_type_and_code_6_to_4(
                    Icmpv6Type(original_payload_inner[0]),
                    Icmpv6Code(original_payload_inner[1]),
                ) {
                    let inner_icmp =
                        icmp_packet!(icmp_type, icmp_code, &original_payload_inner[8..]);
                    original_payload_inner = inner_icmp.packet().to_vec();
                    log::debug!("Translated inner ICMP packet: {:?}", original_payload_inner);
                }
            }

            // Build a new IPv6 packet out of the embedded IPv4 packet's data
            let new_payload_packet = ipv4_packet!(
                new_source,
                new_dest,
                match original_payload.get_next_header() {
                    IpNextHeaderProtocols::Icmpv6 => IpNextHeaderProtocols::Icmp,
                    proto => proto,
                },
                original_payload.get_hop_limit(),
                &original_payload_inner[..std::cmp::min(8, original_payload_inner.len())]
            );

            // Set the payload
            output_payload = vec![0u8; 4 + new_payload_packet.packet().len()];
            output_payload[4..].copy_from_slice(new_payload_packet.packet());
        }

        // Create a new ICMP packet for the translated values to be stored in
        let mut output = MutableIcmpPacket::owned(vec![
            0u8;
            IcmpPacket::minimum_packet_size()
                + output_payload.len()
        ])
        .unwrap();

        // Set the type and code
        output.set_icmp_type(icmp_type);
        output.set_icmp_code(icmp_code);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(icmp::checksum(&output.to_immutable()));

        // Set the payload
        output.set_payload(&output_payload);

        // Return the translated packet
        return Ok(Some(
            IcmpPacket::owned(output.to_immutable().packet().to_vec()).unwrap(),
        ));
    }

    Ok(None)
}
