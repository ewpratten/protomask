use std::net::{Ipv4Addr, Ipv6Addr};

use ipnet::Ipv6Net;

use crate::packet::error::PacketError;

/// Embed an IPv4 address in an IPv6 prefix
pub fn embed_address(ipv4_address: Ipv4Addr, ipv6_prefix: Ipv6Net) -> Ipv6Addr {
    let v4_octets = ipv4_address.octets();
    let v6_octets = ipv6_prefix.addr().octets();
    Ipv6Addr::new(
        u16::from_be_bytes([v6_octets[0], v6_octets[1]]),
        u16::from_be_bytes([v6_octets[2], v6_octets[3]]),
        u16::from_be_bytes([v6_octets[4], v6_octets[5]]),
        u16::from_be_bytes([v6_octets[6], v6_octets[7]]),
        u16::from_be_bytes([v6_octets[8], v6_octets[9]]),
        u16::from_be_bytes([v6_octets[10], v6_octets[11]]),
        u16::from_be_bytes([v4_octets[0], v4_octets[1]]),
        u16::from_be_bytes([v4_octets[2], v4_octets[3]]),
    )
}

/// Extract an IPv4 address from an IPv6 address
pub fn extract_address(ipv6_address: Ipv6Addr) -> Ipv4Addr {
    let octets = ipv6_address.octets();
    Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15])
}

/// Logs errors instead of crashing out of them
pub fn unwrap_log<T>(result: Result<T, PacketError>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(err) => match err {
            PacketError::MismatchedAddressFamily(addr_a, addr_b) => {
                log::error!(
                    "Mismatched address family between {} and {}",
                    addr_a,
                    addr_b
                );
                None
            }
            PacketError::TooShort(len, data) => {
                log::warn!("Received packet that's too short to parse. Length {}", len);
                log::debug!("Short packet: {:?}", data);
                None
            }
            PacketError::UnsupportedIcmpType(icmp_type) => {
                log::warn!("Unsupported ICMP type {}", icmp_type);
                None
            }
            PacketError::UnsupportedIcmpv6Type(icmp_type) => {
                log::warn!("Unsupported ICMPv6 type {}", icmp_type);
                None
            }
        },
    }
}
