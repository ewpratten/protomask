use std::net::Ipv4Addr;

use pnet_packet::{
    ip::IpNextHeaderProtocol,
    ipv4::{Ipv4Option, Ipv4OptionPacket},
    Packet,
};

use crate::net::packet::error::PacketError;

#[derive(Debug, Clone)]
pub struct Ipv4Packet<T> {
    pub dscp: u8,
    pub ecn: u8,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: IpNextHeaderProtocol,
    pub source_address: Ipv4Addr,
    pub destination_address: Ipv4Addr,
    pub options: Vec<Ipv4Option>,
    pub payload: T,
}

impl<T> Ipv4Packet<T> {
    /// Construct a new IPv4 packet
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dscp: u8,
        ecn: u8,
        identification: u16,
        flags: u8,
        fragment_offset: u16,
        ttl: u8,
        protocol: IpNextHeaderProtocol,
        source_address: Ipv4Addr,
        destination_address: Ipv4Addr,
        options: Vec<Ipv4Option>,
        payload: T,
    ) -> Self {
        Self {
            dscp,
            ecn,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            source_address,
            destination_address,
            options,
            payload,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn options_length_words(&self) -> u8 {
        self.options
            .iter()
            .map(|option| Ipv4OptionPacket::packet_size(option) as u8)
            .sum::<u8>()
            / 4
    }
}

impl<T> TryFrom<Vec<u8>> for Ipv4Packet<T>
where
    T: From<Vec<u8>>,
{
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Parse the packet
        let packet = pnet_packet::ipv4::Ipv4Packet::new(&bytes)
            .ok_or(PacketError::TooShort(bytes.len(), bytes.clone()))?;

        // Return the packet
        Ok(Self {
            dscp: packet.get_dscp(),
            ecn: packet.get_ecn(),
            identification: packet.get_identification(),
            flags: packet.get_flags(),
            fragment_offset: packet.get_fragment_offset(),
            ttl: packet.get_ttl(),
            protocol: packet.get_next_level_protocol(),
            source_address: packet.get_source(),
            destination_address: packet.get_destination(),
            options: packet.get_options(),
            payload: packet.payload().to_vec().into(),
        })
    }
}

impl<T> From<Ipv4Packet<T>> for Vec<u8>
where
    T: Into<Vec<u8>> + Clone,
{
    fn from(packet: Ipv4Packet<T>) -> Self {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = packet.payload.clone().into();

        // Build the packet
        let total_length = 20 + (packet.options_length_words() as usize * 4) + payload.len();
        let mut output =
            pnet_packet::ipv4::MutableIpv4Packet::owned(vec![0u8; total_length]).unwrap();

        // Set the fields
        output.set_version(4);
        output.set_header_length(5 + packet.options_length_words());
        output.set_dscp(packet.dscp);
        output.set_ecn(packet.ecn);
        output.set_total_length(total_length.try_into().unwrap());
        output.set_identification(packet.identification);
        output.set_flags(packet.flags);
        output.set_fragment_offset(packet.fragment_offset);
        output.set_ttl(packet.ttl);
        output.set_next_level_protocol(packet.protocol);
        output.set_source(packet.source_address);
        output.set_destination(packet.destination_address);
        output.set_options(&packet.options);

        // Set the payload
        output.set_payload(&payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(pnet_packet::ipv4::checksum(&output.to_immutable()));

        // Return the packet
        output.to_immutable().packet().to_vec()
    }
}
