//! ICMP packets require their own translation system

use std::net::Ipv6Addr;

use colored::Colorize;
use pnet_packet::{
    icmp::{self, Icmp, IcmpCode, IcmpPacket, IcmpType, MutableIcmpPacket},
    icmpv6::{self, Icmpv6Packet, MutableIcmpv6Packet, Icmpv6, Icmpv6Type, Icmpv6Code},
    Packet,
};

pub fn icmpv6_to_icmp<'a>(input: &'a Icmpv6Packet<'a>) -> Option<IcmpPacket<'a>> {
    let data = match input.get_icmpv6_type().0 {
        // Destination Unreachable
        1 => Icmp {
            icmp_type: IcmpType(3),
            // A best guess translation of ICMP codes. Feel free to open a PR to improve this :)
            icmp_code: IcmpCode(match input.get_icmpv6_code().0 {
                // No route to destination -> Destination network unreachable
                0 => 0,
                // Communication with destination administratively prohibited -> Communication administratively prohibited
                1 => 13,
                // Beyond scope of source address -> Destination network unreachable
                2 => 0,
                // Address unreachable -> Destination host unreachable
                3 => 1,
                // Port unreachable -> Destination port unreachable
                4 => 3,
                // Source address failed ingress/egress policy -> Source route failed
                5 => 5,
                // Reject route to destination -> Destination network unreachable
                6 => 0,
                // Error in Source Routing Header -> Destination network unreachable
                7 => 0,
                // All others -> Destination network unreachable
                _ => 0,
            }),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        // Time Exceeded
        3 => Icmp {
            icmp_type: IcmpType(11),
            icmp_code: IcmpCode(input.get_icmpv6_code().0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        // Echo Request
        128 => Icmp {
            icmp_type: IcmpType(8),
            icmp_code: IcmpCode(0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        // Echo Reply
        129 => Icmp {
            icmp_type: IcmpType(0),
            icmp_code: IcmpCode(0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        _ => {
            log::warn!("ICMPv6 type {} not supported", input.get_icmpv6_type().0);
            return None;
        }
    };

    // Debug logging
    #[cfg_attr(rustfmt, rustfmt_skip)]
    {
        log::debug!("> Input ICMP Type: {}", input.get_icmpv6_type().0.to_string().bright_cyan());
        log::debug!("> Input ICMP Code: {}", input.get_icmpv6_code().0.to_string().bright_cyan());
        log::debug!("> Output ICMP Type: {}", data.icmp_type.0.to_string().bright_cyan());
        log::debug!("> Output ICMP Code: {}", data.icmp_code.0.to_string().bright_cyan());
    }

    // Create new ICMP packet
    let mut output = MutableIcmpPacket::owned(vec![0u8; IcmpPacket::packet_size(&data)]).unwrap();
    output.populate(&data);
    output.set_checksum(icmp::checksum(&output.to_immutable()));

    IcmpPacket::owned(output.to_immutable().packet().to_vec())
}

pub fn icmp_to_icmpv6<'a>(
    input: &'a IcmpPacket<'a>,
    source: &Ipv6Addr,
    dest: &Ipv6Addr,
) -> Option<Icmpv6Packet<'a>> {
    let data = match input.get_icmp_type().0 {
        // Destination Unreachable
        3 => Icmpv6 {
            icmpv6_type: Icmpv6Type(1),
            // A best guess translation of ICMP codes. Feel free to open a PR to improve this :)
            icmpv6_code: Icmpv6Code(match input.get_icmp_code().0 {
                // Destination network unreachable -> No route to destination
                0 => 0,
                // Destination host unreachable -> Address unreachable
                1 => 3,
                // Destination protocol unreachable -> No route to destination
                2 => 0,
                // Destination port unreachable -> Port unreachable
                3 => 4,
                // Fragmentation required, and DF flag set -> Packet too big
                4 => 2,
                // Source route failed -> Source address failed ingress/egress policy
                5 => 5,
                // Destination network unknown -> No route to destination
                6 => 0,
                // Destination host unknown -> Address unreachable
                7 => 3,
                // Source host isolated -> No route to destination
                8 => 0,
                // Network administratively prohibited -> Communication with destination administratively prohibited
                9 => 1,
                // Host administratively prohibited -> Communication with destination administratively prohibited
                10 => 1,
                // Network unreachable for ToS -> No route to destination
                11 => 0,
                // Host unreachable for ToS -> Address unreachable
                12 => 3,
                // Communication administratively prohibited -> Communication with destination administratively prohibited
                13 => 1,
                // Host Precedence Violation -> Communication with destination administratively prohibited
                14 => 1,
                // Precedence cutoff in effect -> Communication with destination administratively prohibited
                15 => 1,
                // All others -> No route to destination
                _ => 0,
            }),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        // Time Exceeded
        11 => Icmpv6 {
            icmpv6_type: Icmpv6Type(3),
            icmpv6_code: Icmpv6Code(input.get_icmp_code().0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        // Echo Request
        8 => Icmpv6 {
            icmpv6_type: Icmpv6Type(128),
            icmpv6_code: Icmpv6Code(0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },

        // Echo Reply
        0 => Icmpv6 {
            icmpv6_type: Icmpv6Type(129),
            icmpv6_code: Icmpv6Code(0),
            checksum: 0,
            payload: input.payload().to_vec(),
        },
        _ => {
            log::warn!("ICMP type {} not supported", input.get_icmp_type().0);
            return None;
        }
    };

    // Debug logging
    #[cfg_attr(rustfmt, rustfmt_skip)]
    {
        log::debug!("> Input ICMP Type: {}", input.get_icmp_type().0.to_string().bright_cyan());
        log::debug!("> Input ICMP Code: {}", input.get_icmp_code().0.to_string().bright_cyan());
        log::debug!("> Output ICMP Type: {}", data.icmpv6_type.0.to_string().bright_cyan());
        log::debug!("> Output ICMP Code: {}", data.icmpv6_code.0.to_string().bright_cyan());
    }

    // Create new ICMP packet
    let mut output = MutableIcmpv6Packet::owned(vec![0u8; Icmpv6Packet::packet_size(&data)]).unwrap();
    output.populate(&data);
    output.set_checksum(icmpv6::checksum(&output.to_immutable(), source, dest));

    Icmpv6Packet::owned(output.to_immutable().packet().to_vec())
}
