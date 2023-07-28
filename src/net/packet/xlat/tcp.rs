use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::{
    net::packet::{
        error::PacketError,
        protocols::{raw::RawBytes, tcp::TcpPacket},
    },
    utils::profiling::{PacketTimer, TimerScope},
};

/// Translates an IPv4 TCP packet to an IPv6 TCP packet
pub fn translate_tcp4_to_tcp6(
    input: TcpPacket<RawBytes>,
    new_source_addr: Ipv6Addr,
    new_destination_addr: Ipv6Addr,
    timer: &mut PacketTimer,
) -> Result<TcpPacket<RawBytes>, PacketError> {
    // Build the packet
    timer.start(TimerScope::Tcp);
    let output = TcpPacket::new(
        SocketAddr::new(IpAddr::V6(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V6(new_destination_addr), input.destination().port()),
        input.sequence,
        input.ack_number,
        input.flags,
        input.window_size,
        input.urgent_pointer,
        input.options,
        input.payload,
    );
    timer.end(TimerScope::Tcp);
    output
}

/// Translates an IPv6 TCP packet to an IPv4 TCP packet
pub fn translate_tcp6_to_tcp4(
    input: TcpPacket<RawBytes>,
    new_source_addr: Ipv4Addr,
    new_destination_addr: Ipv4Addr,
    timer: &mut PacketTimer,
) -> Result<TcpPacket<RawBytes>, PacketError> {
    // Build the packet
    timer.start(TimerScope::Tcp);
    let output = TcpPacket::new(
        SocketAddr::new(IpAddr::V4(new_source_addr), input.source().port()),
        SocketAddr::new(IpAddr::V4(new_destination_addr), input.destination().port()),
        input.sequence,
        input.ack_number,
        input.flags,
        input.window_size,
        input.urgent_pointer,
        input.options,
        input.payload,
    );
    timer.end(TimerScope::Tcp);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_tcp4_to_tcp6() {
        let input = TcpPacket::new(
            "192.0.2.1:1234".parse().unwrap(),
            "192.0.2.2:5678".parse().unwrap(),
            123456,
            654321,
            0,
            4096,
            0,
            Vec::new(),
            RawBytes("Hello, world!".as_bytes().to_vec()),
        )
        .unwrap();

        let result = translate_tcp4_to_tcp6(
            input,
            "2001:db8::1".parse().unwrap(),
            "2001:db8::2".parse().unwrap(),
            &mut PacketTimer::new(4),
        )
        .unwrap();

        assert_eq!(result.source(), "[2001:db8::1]:1234".parse().unwrap());
        assert_eq!(result.destination(), "[2001:db8::2]:5678".parse().unwrap());
        assert_eq!(result.sequence, 123456);
        assert_eq!(result.ack_number, 654321);
        assert_eq!(result.flags, 0);
        assert_eq!(result.window_size, 4096);
        assert_eq!(result.urgent_pointer, 0);
        assert_eq!(result.options.len(), 0);
        assert_eq!(
            result.payload,
            RawBytes("Hello, world!".as_bytes().to_vec())
        );
    }

    #[test]
    fn test_translate_tcp6_to_tcp4() {
        let input = TcpPacket::new(
            "[2001:db8::1]:1234".parse().unwrap(),
            "[2001:db8::2]:5678".parse().unwrap(),
            123456,
            654321,
            0,
            4096,
            0,
            Vec::new(),
            RawBytes("Hello, world!".as_bytes().to_vec()),
        )
        .unwrap();

        let result = translate_tcp6_to_tcp4(
            input,
            "192.0.2.1".parse().unwrap(),
            "192.0.2.2".parse().unwrap(),
            &mut PacketTimer::new(6),
        )
        .unwrap();

        assert_eq!(result.source(), "192.0.2.1:1234".parse().unwrap());
        assert_eq!(result.destination(), "192.0.2.2:5678".parse().unwrap());
        assert_eq!(result.sequence, 123456);
        assert_eq!(result.ack_number, 654321);
        assert_eq!(result.flags, 0);
        assert_eq!(result.window_size, 4096);
        assert_eq!(result.urgent_pointer, 0);
        assert_eq!(result.options.len(), 0);
        assert_eq!(
            result.payload,
            RawBytes("Hello, world!".as_bytes().to_vec())
        );
    }
}
