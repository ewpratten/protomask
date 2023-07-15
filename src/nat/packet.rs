use std::net::{Ipv4Addr, Ipv6Addr};

use super::utils::ipv4_header_checksum;

/// Constructs an IPv4 packet
pub fn make_ipv4_packet(
    ttl: u8,
    protocol: u8,
    source: &Ipv4Addr,
    destination: &Ipv4Addr,
    payload: &[u8],
) -> Vec<u8> {
    // Allocate an empty buffer
    let mut buffer = vec![0; 20 + payload.len()];

    // Write version and header length
    buffer[0] = 0x45;

    // DSCP and ECN
    let dscp = 0u8;
    let ecn = 0u8;
    buffer[1] = (dscp << 2) | ecn;

    buffer[2] = (buffer.len() >> 8) as u8; // Total length
    buffer[3] = buffer.len() as u8; // Total length (contd.)
    buffer[4] = 0x00; // Identification
    buffer[6] = 0x00; // Flags and fragment offset
    buffer[7] = 0x00; // Fragment offset (contd.)
    buffer[8] = ttl; // TTL
    buffer[9] = protocol; // Protocol
    buffer[10] = 0x00; // Header checksum
    buffer[11] = 0x00; // Header checksum (contd.)
    buffer[12..16].copy_from_slice(&source.octets()); // Source address
    buffer[16..20].copy_from_slice(&destination.octets()); // Destination address

    // Calculate the checksum
    let checksum = ipv4_header_checksum(&buffer[0..20]);
    buffer[10] = (checksum >> 8) as u8;
    buffer[11] = checksum as u8;

    // Copy the payload
    buffer[20..].copy_from_slice(payload);

    // Return the buffer
    buffer
}

pub fn make_ipv6_packet(
    hop_limit: u8,
    next_header: u8,
    source: &Ipv6Addr,
    destination: &Ipv6Addr,
    payload: &[u8],
) -> Vec<u8> {
    // Allocate an empty buffer
    let mut buffer = vec![0; 40 + payload.len()];

    // Write basic info
    buffer[0] = 0x60; // Version and traffic class
    buffer[1] = 0x00; // Traffic class (contd.) and flow label
    buffer[2] = 0x00; // Flow label (contd.)
    buffer[3] = 0x00; // Flow label (contd.)
    buffer[4] = (buffer.len() >> 8) as u8; // Payload length
    buffer[5] = buffer.len() as u8; // Payload length (contd.)
    buffer[6] = next_header; // Next header
    buffer[7] = hop_limit; // Hop limit
    buffer[8..24].copy_from_slice(&source.octets()); // Source address
    buffer[24..40].copy_from_slice(&destination.octets()); // Destination address

    // Copy the payload
    buffer[40..].copy_from_slice(payload);

    // Return the buffer
    buffer
}
