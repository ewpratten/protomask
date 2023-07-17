/// Quickly convert a byte slice into an ICMP packet
#[macro_export]
macro_rules! into_icmp {
    ($bytes:expr) => {
        pnet_packet::icmp::IcmpPacket::owned($bytes).ok_or_else(|| {
            crate::nat::xlat::PacketTranslationError::InputPacketTooShort($bytes.len())
        })
    };
}

/// Quickly convert a byte slice into an ICMPv6 packet
#[macro_export]
macro_rules! into_icmpv6 {
    ($bytes:expr) => {
        pnet_packet::icmpv6::Icmpv6Packet::owned($bytes).ok_or_else(|| {
            crate::nat::xlat::PacketTranslationError::InputPacketTooShort($bytes.len())
        })
    };
}

/// Quickly convert a byte slice into a UDP packet
#[macro_export]
macro_rules! into_udp {
    ($bytes:expr) => {
        pnet_packet::udp::UdpPacket::owned($bytes).ok_or_else(|| {
            crate::nat::xlat::PacketTranslationError::InputPacketTooShort($bytes.len())
        })
    };
}

/// Quickly convert a byte slice into a TCP packet
#[macro_export]
macro_rules! into_tcp {
    ($bytes:expr) => {
        pnet_packet::tcp::TcpPacket::owned($bytes).ok_or_else(|| {
            crate::nat::xlat::PacketTranslationError::InputPacketTooShort($bytes.len())
        })
    };
}

/// Quickly construct an IPv6 packet with the given parameters
#[macro_export]
macro_rules! ipv6_packet {
    ($source:expr, $destination:expr, $next_header:expr, $hop_limit:expr, $payload:expr) => {
        ipv6_packet!(
            $source,
            $destination,
            0,
            0,
            $next_header,
            $hop_limit,
            $payload
        )
    };

    ($source:expr, $destination:expr, $traffic_class:expr, $flow_label:expr, $next_header:expr, $hop_limit:expr, $payload:expr) => {{
        let mut output =
            pnet_packet::ipv6::MutableIpv6Packet::owned(vec![0u8; 40 + $payload.len()]).unwrap();
        output.set_version(6);
        output.set_traffic_class($traffic_class);
        output.set_flow_label($flow_label);
        output.set_next_header($next_header);
        output.set_hop_limit($hop_limit);
        output.set_source($source);
        output.set_destination($destination);
        output.set_payload_length($payload.len() as u16);
        output.set_payload($payload);
        pnet_packet::ipv6::Ipv6Packet::owned(output.to_immutable().packet().to_vec()).unwrap()
    }};
}

/// Quickly construct an IPv4 packet with the given parameters
#[macro_export]
macro_rules! ipv4_packet {
    ($source:expr, $destination:expr, $ttl:expr, $next_level_protocol:expr, $payload:expr) => {
        ipv4_packet!(
            $source,
            $destination,
            0,
            0,
            0,
            0,
            0,
            $ttl,
            $next_level_protocol,
            // &[],
            $payload
        )
    };

    // NOTE: Temporarily disabled options, since we aren't using them
    // ($source:expr, $destination:expr, $dscp:expr, $ecn:expr, $identification:expr, $flags:expr, $fragment_offset:expr, $ttl:expr, $next_level_protocol:expr, $options:expr, $payload:expr) => {{
    ($source:expr, $destination:expr, $dscp:expr, $ecn:expr, $identification:expr, $flags:expr, $fragment_offset:expr, $ttl:expr, $next_level_protocol:expr,  $payload:expr) => {{
        // let total_option_length = $options
        //     .iter()
        //     .map(|o: pnet_packet::ipv4::Ipv4Option| pnet_packet::Packet::payload(o).len())
        //     .sum::<usize>();
        let total_option_length: usize = 0;
        let mut output = pnet_packet::ipv4::MutableIpv4Packet::owned(vec![
            0u8;
            20 + total_option_length
                + $payload.len()
        ])
        .unwrap();
        output.set_version(4);
        output.set_header_length(((20 + total_option_length) / (32 / 8)) as u8); // Dynamic header length :(
        output.set_dscp($dscp);
        output.set_ecn($ecn);
        output.set_total_length((20 + total_option_length + $payload.len()) as u16);
        output.set_identification($identification);
        output.set_flags($flags);
        output.set_fragment_offset($fragment_offset);
        output.set_ttl($ttl);
        output.set_next_level_protocol($next_level_protocol);
        output.set_source($source);
        output.set_destination($destination);
        // output.set_options($options);
        output.set_payload($payload);
        output.set_checksum(0);
        output.set_checksum(pnet_packet::ipv4::checksum(&output.to_immutable()));
        pnet_packet::ipv4::Ipv4Packet::owned(output.to_immutable().packet().to_vec()).unwrap()
    }};
}
