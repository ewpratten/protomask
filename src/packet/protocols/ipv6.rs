use std::net::Ipv6Addr;

use pnet_packet::{ip::IpNextHeaderProtocol, Packet};

use crate::packet::error::PacketError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ipv6Packet<T> {
    pub traffic_class: u8,
    pub flow_label: u32,
    pub next_header: IpNextHeaderProtocol,
    pub hop_limit: u8,
    pub source_address: Ipv6Addr,
    pub destination_address: Ipv6Addr,
    pub payload: T,
}

impl<T> Ipv6Packet<T> {
    /// Construct a new IPv6 packet
    pub fn new(
        traffic_class: u8,
        flow_label: u32,
        next_header: IpNextHeaderProtocol,
        hop_limit: u8,
        source_address: Ipv6Addr,
        destination_address: Ipv6Addr,
        payload: T,
    ) -> Self {
        Self {
            traffic_class,
            flow_label,
            next_header,
            hop_limit,
            source_address,
            destination_address,
            payload,
        }
    }
}

impl<T> TryFrom<Vec<u8>> for Ipv6Packet<T>
where
    T: From<Vec<u8>>,
{
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Parse the packet
        let packet = pnet_packet::ipv6::Ipv6Packet::new(&bytes)
            .ok_or(PacketError::TooShort(bytes.len(), bytes.clone()))?;

        // Return the packet
        Ok(Self {
            traffic_class: packet.get_traffic_class(),
            flow_label: packet.get_flow_label(),
            next_header: packet.get_next_header(),
            hop_limit: packet.get_hop_limit(),
            source_address: packet.get_source(),
            destination_address: packet.get_destination(),
            payload: packet.payload().to_vec().into(),
        })
    }
}

impl<T> From<Ipv6Packet<T>> for Vec<u8>
where
    T: Into<Vec<u8>>,
{
    fn from(packet: Ipv6Packet<T>) -> Self {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = packet.payload.into();

        // Allocate a mutable packet to write into
        let total_length =
            pnet_packet::ipv6::MutableIpv6Packet::minimum_packet_size() + payload.len();
        let mut output =
            pnet_packet::ipv6::MutableIpv6Packet::owned(vec![0u8; total_length]).unwrap();

        // Write the header
        output.set_version(6);
        output.set_traffic_class(packet.traffic_class);
        output.set_flow_label(packet.flow_label);
        output.set_payload_length(u16::try_from(payload.len()).unwrap());
        output.set_next_header(packet.next_header);
        output.set_hop_limit(packet.hop_limit);
        output.set_source(packet.source_address);
        output.set_destination(packet.destination_address);

        // Write the payload
        output.set_payload(&payload);

        // Return the packet
        output.to_immutable().packet().to_vec()
    }
}
