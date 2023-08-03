use std::net::{Ipv4Addr, Ipv6Addr};

use pnet::packet::udp::{self, MutableUdpPacket, UdpPacket};

use crate::error::{Error, Result};

/// Re-calculates a UDP packet's checksum with a new IPv6 pseudo-header.
pub fn recalculate_udp_checksum_ipv6(
    udp_packet: &[u8],
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Clone the packet so we can modify it
        let mut udp_packet_buffer = udp_packet.to_vec();

        // Get safe mutable access to the packet
        let mut udp_packet =
            MutableUdpPacket::new(&mut udp_packet_buffer).ok_or(Error::PacketTooShort {
                expected: UdpPacket::minimum_packet_size(),
                actual: udp_packet.len(),
            })?;

        // Edit the packet's checksum
        udp_packet.set_checksum(0);
        udp_packet.set_checksum(udp::ipv6_checksum(
            &udp_packet.to_immutable(),
            &new_source,
            &new_destination,
        ));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_UDP, STATUS_TRANSLATED).inc();

        // Return the translated packet
        Ok(udp_packet_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_UDP, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

/// Re-calculates a UDP packet's checksum with a new IPv4 pseudo-header.
pub fn recalculate_udp_checksum_ipv4(
    udp_packet: &[u8],
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Clone the packet so we can modify it
        let mut udp_packet_buffer = udp_packet.to_vec();

        // Get safe mutable access to the packet
        let mut udp_packet =
            MutableUdpPacket::new(&mut udp_packet_buffer).ok_or(Error::PacketTooShort {
                expected: UdpPacket::minimum_packet_size(),
                actual: udp_packet.len(),
            })?;

        // Edit the packet's checksum
        udp_packet.set_checksum(0);
        udp_packet.set_checksum(udp::ipv4_checksum(
            &udp_packet.to_immutable(),
            &new_source,
            &new_destination,
        ));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_UDP, STATUS_TRANSLATED).inc();

        // Return the translated packet
        Ok(udp_packet_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_UDP, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recalculate_udp_checksum_ipv6() {
        let mut input_buffer = vec![0u8; UdpPacket::minimum_packet_size() + 13];
        let mut udp_packet = MutableUdpPacket::new(&mut input_buffer).unwrap();
        udp_packet.set_source(1234);
        udp_packet.set_destination(5678);
        udp_packet.set_length(13);
        udp_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

        // Recalculate the checksum
        let recalculated_buffer = recalculate_udp_checksum_ipv6(
            &input_buffer,
            "2001:db8::1".parse().unwrap(),
            "2001:db8::2".parse().unwrap(),
        )
        .unwrap();

        // Check that the checksum is correct
        let recalculated_packet = UdpPacket::new(&recalculated_buffer).unwrap();
        assert_eq!(recalculated_packet.get_checksum(), 0x480b);
    }

    #[test]
    fn test_recalculate_udp_checksum_ipv4() {
        let mut input_buffer = vec![0u8; UdpPacket::minimum_packet_size() + 13];
        let mut udp_packet = MutableUdpPacket::new(&mut input_buffer).unwrap();
        udp_packet.set_source(1234);
        udp_packet.set_destination(5678);
        udp_packet.set_length(13);
        udp_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

        // Recalculate the checksum
        let recalculated_buffer = recalculate_udp_checksum_ipv4(
            &input_buffer,
            "192.0.2.1".parse().unwrap(),
            "192.0.2.2".parse().unwrap(),
        )
        .unwrap();

        // Check that the checksum is correct
        let recalculated_packet = UdpPacket::new(&recalculated_buffer).unwrap();
        assert_eq!(recalculated_packet.get_checksum(), 0x1f7c);
    }
}
