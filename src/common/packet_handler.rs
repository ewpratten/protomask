use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, thiserror::Error)]
pub enum PacketHandlingError {
    #[error(transparent)]
    InterprotoError(#[from] interproto::error::Error),
    #[error(transparent)]
    FastNatError(#[from] fast_nat::error::Error),
}

/// Handles checking the version number of an IP packet and calling the correct handler with needed data
pub fn handle_packet<Ipv4Handler, Ipv6Handler>(
    packet: &[u8],
    mut ipv4_handler: Ipv4Handler,
    mut ipv6_handler: Ipv6Handler,
) -> Option<Vec<u8>>
where
    Ipv4Handler: FnMut(&[u8], &Ipv4Addr, &Ipv4Addr) -> Result<Option<Vec<u8>>, PacketHandlingError>,
    Ipv6Handler: FnMut(&[u8], &Ipv6Addr, &Ipv6Addr) -> Result<Option<Vec<u8>>, PacketHandlingError>,
{
    // If the packet is empty, return nothing
    if packet.is_empty() {
        return None;
    }

    // Switch on the layer 3 protocol number to call the correct handler
    let layer_3_proto = packet[0] >> 4;
    log::trace!("New packet with layer 3 protocol: {}", layer_3_proto);
    let handler_response = match layer_3_proto {
        // IPv4
        4 => {
            // Extract the source and destination addresses
            let source_addr =
                Ipv4Addr::from(u32::from_be_bytes(packet[12..16].try_into().unwrap()));
            let destination_addr =
                Ipv4Addr::from(u32::from_be_bytes(packet[16..20].try_into().unwrap()));

            // Call the handler
            ipv4_handler(packet, &source_addr, &destination_addr)
        }

        // IPv6
        6 => {
            // Extract the source and destination addresses
            let source_addr =
                Ipv6Addr::from(u128::from_be_bytes(packet[8..24].try_into().unwrap()));
            let destination_addr =
                Ipv6Addr::from(u128::from_be_bytes(packet[24..40].try_into().unwrap()));

            // Call the handler
            ipv6_handler(packet, &source_addr, &destination_addr)
        }

        // Unknown protocol numbers can't be handled
        proto => {
            log::warn!("Unknown Layer 3 protocol: {}", proto);
            return None;
        }
    };

    // The response from the handler may or may not be a warn-able error
    match handler_response {
        // If we get data, return it
        Ok(data) => data,
        // If we get an error, handle it and return None
        Err(error) => match error {
            PacketHandlingError::InterprotoError(interproto::error::Error::PacketTooShort {
                expected,
                actual,
            }) => {
                log::warn!(
                    "Got packet with length {} when expecting at least {} bytes",
                    actual,
                    expected
                );
                None
            }
            PacketHandlingError::InterprotoError(
                interproto::error::Error::UnsupportedIcmpType(icmp_type),
            ) => {
                log::warn!("Got a packet with an unsupported ICMP type: {}", icmp_type);
                None
            }
            PacketHandlingError::InterprotoError(
                interproto::error::Error::UnsupportedIcmpv6Type(icmpv6_type),
            ) => {
                log::warn!(
                    "Got a packet with an unsupported ICMPv6 type: {}",
                    icmpv6_type
                );
                None
            }
            PacketHandlingError::FastNatError(fast_nat::error::Error::Ipv4PoolExhausted(size)) => {
                log::warn!("IPv4 pool exhausted with {} mappings", size);
                None
            }
            PacketHandlingError::FastNatError(fast_nat::error::Error::InvalidIpv4Address(addr)) => {
                log::warn!("Invalid IPv4 address: {}", addr);
                None
            }
        },
    }
}
