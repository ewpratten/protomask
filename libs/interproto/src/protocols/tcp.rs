use std::net::{Ipv4Addr, Ipv6Addr};

use pnet::packet::tcp::{self, MutableTcpPacket, TcpPacket};

use crate::error::{Error, Result};

/// Re-calculates a TCP packet's checksum with a new IPv6 pseudo-header.
#[profiling::function]
pub fn recalculate_tcp_checksum_ipv6(
    tcp_packet: &[u8],
    new_source: Ipv6Addr,
    new_destination: Ipv6Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Clone the packet so we can modify it
        let mut tcp_packet_buffer = tcp_packet.to_vec();

        // Get safe mutable access to the packet
        let mut tcp_packet =
            MutableTcpPacket::new(&mut tcp_packet_buffer).ok_or(Error::PacketTooShort {
                expected: TcpPacket::minimum_packet_size(),
                actual: tcp_packet.len(),
            })?;

        // Edit the packet's checksum
        tcp_packet.set_checksum(0);
        tcp_packet.set_checksum(tcp::ipv6_checksum(
            &tcp_packet.to_immutable(),
            &new_source,
            &new_destination,
        ));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_TCP, STATUS_TRANSLATED).inc();

        // Return the translated packet
        Ok(tcp_packet_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_TCP, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

/// Re-calculates a TCP packet's checksum with a new IPv4 pseudo-header.
#[profiling::function]
pub fn recalculate_tcp_checksum_ipv4(
    tcp_packet: &[u8],
    new_source: Ipv4Addr,
    new_destination: Ipv4Addr,
) -> Result<Vec<u8>> {
    // This scope is used to collect packet drop metrics
    {
        // Clone the packet so we can modify it
        let mut tcp_packet_buffer = tcp_packet.to_vec();

        // Get safe mutable access to the packet
        let mut tcp_packet =
            MutableTcpPacket::new(&mut tcp_packet_buffer).ok_or(Error::PacketTooShort {
                expected: TcpPacket::minimum_packet_size(),
                actual: tcp_packet.len(),
            })?;

        // Edit the packet's checksum
        tcp_packet.set_checksum(0);
        tcp_packet.set_checksum(tcp::ipv4_checksum(
            &tcp_packet.to_immutable(),
            &new_source,
            &new_destination,
        ));

        // Track the translated packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_TCP, STATUS_TRANSLATED).inc();

        // Return the translated packet
        Ok(tcp_packet_buffer)
    }
    .map_err(|error| {
        // Track the dropped packet
        #[cfg(feature = "metrics")]
        protomask_metrics::metric!(PACKET_COUNTER, PROTOCOL_TCP, STATUS_DROPPED).inc();

        // Pass the error through
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_recalculate_ipv6() {
        // Create an input packet
        let mut input_buffer = vec![0u8; TcpPacket::minimum_packet_size() + 13];
        let mut input_packet = MutableTcpPacket::new(&mut input_buffer).unwrap();
        input_packet.set_source(1234);
        input_packet.set_destination(5678);
        input_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

        // Recalculate the checksum
        let recalculated_buffer = recalculate_tcp_checksum_ipv6(
            &input_buffer,
            "2001:db8::1".parse().unwrap(),
            "2001:db8::2".parse().unwrap(),
        )
        .unwrap();

        // Verify the checksum
        let recalculated_packet = TcpPacket::new(&recalculated_buffer).unwrap();
        assert_eq!(recalculated_packet.get_checksum(), 0x4817);
    }

    #[test]
    fn test_checksum_recalculate_ipv4() {
        // Create an input packet
        let mut input_buffer = vec![0u8; TcpPacket::minimum_packet_size() + 13];
        let mut input_packet = MutableTcpPacket::new(&mut input_buffer).unwrap();
        input_packet.set_source(1234);
        input_packet.set_destination(5678);
        input_packet.set_payload(&"Hello, world!".as_bytes().to_vec());

        // Recalculate the checksum
        let recalculated_buffer = recalculate_tcp_checksum_ipv4(
            &input_buffer,
            "192.0.2.1".parse().unwrap(),
            "192.0.2.2".parse().unwrap(),
        )
        .unwrap();

        // Verify the checksum
        let recalculated_packet = TcpPacket::new(&recalculated_buffer).unwrap();
        assert_eq!(recalculated_packet.get_checksum(), 0x1f88);
    }
}
