use std::net::Ipv4Addr;

use pnet_packet::{
    ip::IpNextHeaderProtocol,
    ipv4::{Ipv4Option, Ipv4OptionPacket},
    Packet,
};

use crate::packet::error::PacketError;

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
        let packet =
            pnet_packet::ipv4::Ipv4Packet::new(&bytes).ok_or(PacketError::TooShort(bytes.len(), bytes.to_vec()))?;

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

impl<T> Into<Vec<u8>> for Ipv4Packet<T>
where
    T: Into<Vec<u8>> + Clone,
{
    fn into(self) -> Vec<u8> {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = self.payload.clone().into();

        // Build the packet
        let total_length = 20 + (self.options_length_words() as usize * 4) + payload.len();
        let mut packet =
            pnet_packet::ipv4::MutableIpv4Packet::owned(vec![0u8; total_length]).unwrap();

        // Set the fields
        packet.set_version(4);
        packet.set_header_length(5 + self.options_length_words());
        packet.set_dscp(self.dscp);
        packet.set_ecn(self.ecn);
        packet.set_total_length(total_length.try_into().unwrap());
        packet.set_identification(self.identification);
        packet.set_flags(self.flags);
        packet.set_fragment_offset(self.fragment_offset);
        packet.set_ttl(self.ttl);
        packet.set_next_level_protocol(self.protocol);
        packet.set_source(self.source_address);
        packet.set_destination(self.destination_address);
        packet.set_options(&self.options);

        // Set the payload
        packet.set_payload(&payload);

        // Calculate the checksum
        packet.set_checksum(0);
        packet.set_checksum(pnet_packet::ipv4::checksum(&packet.to_immutable()));

        // Return the packet
        packet.to_immutable().packet().to_vec()
    }
}
