use std::net::{Ipv4Addr, Ipv6Addr};

use ipnet::Ipv6Net;

/// Calculates the checksum value for an IPv4 header
pub fn ipv4_header_checksum(header: &[u8]) -> u16 {
    let mut sum = 0u32;

    // Iterate over the header in 16-bit chunks
    for i in (0..header.len()).step_by(2) {
        // Combine the two bytes into a 16-bit integer
        let word = ((header[i] as u16) << 8) | (header[i + 1] as u16);

        // Add to the sum
        sum = sum.wrapping_add(word as u32);
    }

    // Fold the carry bits
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    // Return the checksum
    !(sum as u16)
}

/// Convert bytes to an IPv6 address
pub fn bytes_to_ipv6_addr(bytes: &[u8]) -> Ipv6Addr {
    assert!(bytes.len() == 16);
    let mut octets = [0u8; 16];
    octets.copy_from_slice(bytes);
    Ipv6Addr::from(octets)
}

/// Convert bytes to an IPv4 address
pub fn bytes_to_ipv4_addr(bytes: &[u8]) -> Ipv4Addr {
    assert!(bytes.len() == 4);
    let mut octets = [0u8; 4];
    octets.copy_from_slice(bytes);
    Ipv4Addr::from(octets)
}

/// Converts bytes to a hex string for debugging
pub fn bytes_to_hex_str(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|val| format!("{:02x}", val))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Calculate the appropriate IPv6 address that maps to an IPv4 address
pub fn ipv4_to_ipv6(v4: &Ipv4Addr, prefix: &Ipv6Net) -> Ipv6Addr {
    let net_addr_bytes = prefix.network().octets();
    let v4_bytes = v4.octets();
    return Ipv6Addr::new(
        u16::from_be_bytes([net_addr_bytes[0], net_addr_bytes[1]]),
        u16::from_be_bytes([net_addr_bytes[2], net_addr_bytes[3]]),
        u16::from_be_bytes([net_addr_bytes[4], net_addr_bytes[5]]),
        u16::from_be_bytes([net_addr_bytes[6], net_addr_bytes[7]]),
        u16::from_be_bytes([net_addr_bytes[8], net_addr_bytes[9]]),
        u16::from_be_bytes([net_addr_bytes[10], net_addr_bytes[11]]),
        u16::from_be_bytes([v4_bytes[0], v4_bytes[1]]),
        u16::from_be_bytes([v4_bytes[2], v4_bytes[3]]),
    );
}
