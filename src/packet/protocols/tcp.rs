use std::net::{IpAddr, SocketAddr};

use pnet_packet::{
    tcp::{TcpOption, TcpOptionPacket},
    Packet,
};

use super::raw::RawBytes;
use crate::packet::error::PacketError;

/// A TCP packet
#[derive(Debug, Clone)]
pub struct TcpPacket<T> {
    source: SocketAddr,
    destination: SocketAddr,
    pub sequence: u32,
    pub ack_number: u32,
    pub flags: u8,
    pub window_size: u16,
    pub urgent_pointer: u16,
    pub options: Vec<TcpOption>,
    pub payload: T,
}

impl<T> TcpPacket<T> {
    /// Construct a new TCP packet
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source: SocketAddr,
        destination: SocketAddr,
        sequence: u32,
        ack_number: u32,
        flags: u8,
        window_size: u16,
        urgent_pointer: u16,
        options: Vec<TcpOption>,
        payload: T,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source.is_ipv4() != destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source.ip(),
                destination.ip(),
            ));
        }

        // Build the packet
        Ok(Self {
            source,
            destination,
            sequence,
            ack_number,
            flags,
            window_size,
            urgent_pointer,
            options,
            payload,
        })
    }

    // Set a new source
    #[allow(dead_code)]
    pub fn set_source(&mut self, source: SocketAddr) -> Result<(), PacketError> {
        // Ensure the source and destination addresses are the same type
        if source.is_ipv4() != self.destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source.ip(),
                self.destination.ip(),
            ));
        }

        // Set the source
        self.source = source;

        Ok(())
    }

    // Set a new destination
    #[allow(dead_code)]
    pub fn set_destination(&mut self, destination: SocketAddr) -> Result<(), PacketError> {
        // Ensure the source and destination addresses are the same type
        if self.source.is_ipv4() != destination.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                self.source.ip(),
                destination.ip(),
            ));
        }

        // Set the destination
        self.destination = destination;

        Ok(())
    }

    /// Get the source
    pub fn source(&self) -> SocketAddr {
        self.source
    }

    /// Get the destination
    pub fn destination(&self) -> SocketAddr {
        self.destination
    }

    /// Get the length of the options in words
    #[allow(clippy::cast_possible_truncation)]
    fn options_length(&self) -> u8 {
        self.options
            .iter()
            .map(|option| TcpOptionPacket::packet_size(option) as u8)
            .sum::<u8>()
    }
}

impl<T> TcpPacket<T>
where
    T: From<Vec<u8>>,
{
    /// Construct a new TCP packet from bytes
    #[allow(dead_code)]
    pub fn new_from_bytes(
        bytes: &[u8],
        source_address: IpAddr,
        destination_address: IpAddr,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source_address.is_ipv4() != destination_address.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source_address,
                destination_address,
            ));
        }

        // Parse the packet
        let parsed = pnet_packet::tcp::TcpPacket::new(bytes)
            .ok_or_else(|| PacketError::TooShort(bytes.len(), bytes.to_vec()))?;

        // Build the struct
        Ok(Self {
            source: SocketAddr::new(source_address, parsed.get_source()),
            destination: SocketAddr::new(destination_address, parsed.get_destination()),
            sequence: parsed.get_sequence(),
            ack_number: parsed.get_acknowledgement(),
            flags: parsed.get_flags(),
            window_size: parsed.get_window(),
            urgent_pointer: parsed.get_urgent_ptr(),
            options: parsed.get_options().clone(),
            payload: parsed.payload().to_vec().into(),
        })
    }
}

impl TcpPacket<RawBytes> {
    /// Construct a new TCP packet with a raw payload from bytes
    pub fn new_from_bytes_raw_payload(
        bytes: &[u8],
        source_address: IpAddr,
        destination_address: IpAddr,
    ) -> Result<Self, PacketError> {
        // Ensure the source and destination addresses are the same type
        if source_address.is_ipv4() != destination_address.is_ipv4() {
            return Err(PacketError::MismatchedAddressFamily(
                source_address,
                destination_address,
            ));
        }

        // Parse the packet
        let parsed = pnet_packet::tcp::TcpPacket::new(bytes)
            .ok_or_else(|| PacketError::TooShort(bytes.len(), bytes.to_vec()))?;

        // Build the struct
        Ok(Self {
            source: SocketAddr::new(source_address, parsed.get_source()),
            destination: SocketAddr::new(destination_address, parsed.get_destination()),
            sequence: parsed.get_sequence(),
            ack_number: parsed.get_acknowledgement(),
            flags: parsed.get_flags(),
            window_size: parsed.get_window(),
            urgent_pointer: parsed.get_urgent_ptr(),
            options: parsed.get_options().clone(),
            payload: RawBytes(parsed.payload().to_vec()),
        })
    }
}

impl<T> From<TcpPacket<T>> for Vec<u8>
where
    T: Into<Vec<u8>>,
{
    fn from(packet: TcpPacket<T>) -> Self {
        // Get the options length in words
        let options_length = packet.options_length();

        // Convert the payload into raw bytes
        let payload: Vec<u8> = packet.payload.into();

        // Allocate a mutable packet to write into
        let total_length = pnet_packet::tcp::MutableTcpPacket::minimum_packet_size()
            + options_length as usize
            + payload.len();
        let mut output =
            pnet_packet::tcp::MutableTcpPacket::owned(vec![0u8; total_length]).unwrap();

        // Write the source and dest ports
        output.set_source(packet.source.port());
        output.set_destination(packet.destination.port());

        // Write the sequence and ack numbers
        output.set_sequence(packet.sequence);
        output.set_acknowledgement(packet.ack_number);

        // Write the offset
        output.set_data_offset(5 + (options_length / 4));

        // Write the options
        output.set_options(&packet.options);

        // Write the flags
        output.set_flags(packet.flags);

        // Write the window size
        output.set_window(packet.window_size);

        // Write the urgent pointer
        output.set_urgent_ptr(packet.urgent_pointer);

        // Write the payload
        output.set_payload(&payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(match (packet.source.ip(), packet.destination.ip()) {
            (IpAddr::V4(source_ip), IpAddr::V4(destination_ip)) => {
                pnet_packet::tcp::ipv4_checksum(&output.to_immutable(), &source_ip, &destination_ip)
            }
            (IpAddr::V6(source_ip), IpAddr::V6(destination_ip)) => {
                pnet_packet::tcp::ipv6_checksum(&output.to_immutable(), &source_ip, &destination_ip)
            }
            _ => unreachable!(),
        });

        // Return the raw bytes
        output.packet().to_vec()
    }
}
