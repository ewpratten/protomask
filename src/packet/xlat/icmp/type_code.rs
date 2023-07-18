//! Functions to map between ICMP and ICMPv6 types/codes

use pnet_packet::{
    icmp::{destination_unreachable, IcmpCode, IcmpType, IcmpTypes},
    icmpv6::{Icmpv6Code, Icmpv6Type, Icmpv6Types},
};

use crate::packet::error::PacketError;

/// Best effort translation from an ICMP type and code to an ICMPv6 type and code
pub fn translate_type_and_code_4_to_6(
    icmp_type: IcmpType,
    icmp_code: IcmpCode,
) -> Result<(Icmpv6Type, Icmpv6Code), PacketError> {
    match (icmp_type, icmp_code) {
        // Echo Request
        (IcmpTypes::EchoRequest, _) => Ok((Icmpv6Types::EchoRequest, Icmpv6Code(0))),

        // Echo Reply
        (IcmpTypes::EchoReply, _) => Ok((Icmpv6Types::EchoReply, Icmpv6Code(0))),

        // Packet Too Big
        (
            IcmpTypes::DestinationUnreachable,
            destination_unreachable::IcmpCodes::FragmentationRequiredAndDFFlagSet,
        ) => Ok((Icmpv6Types::PacketTooBig, Icmpv6Code(0))),

        // Destination Unreachable
        (IcmpTypes::DestinationUnreachable, icmp_code) => Ok((
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
            Ok((Icmpv6Types::TimeExceeded, Icmpv6Code(icmp_code.0)))
        }

        // Default unsupported
        (icmp_type, _) => Err(PacketError::UnsupportedIcmpType(icmp_type.0)),
    }
}

/// Best effort translation from an ICMPv6 type and code to an ICMP type and code
pub fn translate_type_and_code_6_to_4(
    icmp_type: Icmpv6Type,
    icmp_code: Icmpv6Code,
) -> Result<(IcmpType, IcmpCode), PacketError> {
    match (icmp_type, icmp_code) {
        // Echo Request
        (Icmpv6Types::EchoRequest, _) => Ok((IcmpTypes::EchoRequest, IcmpCode(0))),

        // Echo Reply
        (Icmpv6Types::EchoReply, _) => Ok((IcmpTypes::EchoReply, IcmpCode(0))),

        // Packet Too Big
        (Icmpv6Types::PacketTooBig, _) => Ok((
            IcmpTypes::DestinationUnreachable,
            destination_unreachable::IcmpCodes::FragmentationRequiredAndDFFlagSet,
        )),

        // Destination Unreachable
        (Icmpv6Types::DestinationUnreachable, icmp_code) => Ok((
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
            Ok((IcmpTypes::TimeExceeded, IcmpCode(icmp_code.0)))
        }

        // Default unsupported
        (icmp_type, _) => Err(PacketError::UnsupportedIcmpv6Type(icmp_type.0)),
    }
}
