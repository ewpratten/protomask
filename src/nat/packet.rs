use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// // use etherparse::{IpHeader, Ipv4Header, Ipv4Extensions};
use pnet_packet::{
    ethernet::EtherTypes::Ipv6,
    ipv4::{checksum, Ipv4, Ipv4Packet, MutableIpv4Packet},
    ipv6::{Ipv6, Ipv6Packet, MutableIpv6Packet},
    Packet,
};

/// A protocol-agnostic packet type
#[derive(Debug)]
pub enum IpPacket<'a> {
    /// IPv4 packet
    V4(Ipv4Packet<'a>),
    /// IPv6 packet
    V6(Ipv6Packet<'a>),
}

impl IpPacket<'_> {
    /// Creates a new packet from a byte slice
    pub fn new<'a>(bytes: &'a [u8]) -> Option<IpPacket<'a>> {
        match bytes[0] >> 4 {
            4 => Some(IpPacket::V4(Ipv4Packet::new(bytes)?)),
            6 => Some(IpPacket::V6(Ipv6Packet::new(bytes)?)),
            _ => None,
        }
    }

    /// Returns the source address
    pub fn get_source(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_source()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_source()),
        }
    }

    /// Returns the destination address
    pub fn get_destination(&self) -> IpAddr {
        match self {
            IpPacket::V4(packet) => IpAddr::V4(packet.get_destination()),
            IpPacket::V6(packet) => IpAddr::V6(packet.get_destination()),
        }
    }

    /// Returns the packet header
    pub fn get_header(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.packet()[..20].as_ref(),
            IpPacket::V6(packet) => packet.packet()[..40].as_ref(),
        }
    }

    /// Returns the packet payload
    pub fn get_payload(&self) -> &[u8] {
        match self {
            IpPacket::V4(packet) => packet.payload(),
            IpPacket::V6(packet) => packet.payload(),
        }
    }

    /// Converts the packet to a byte vector
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IpPacket::V4(packet) => packet.packet().to_vec(),
            IpPacket::V6(packet) => packet.packet().to_vec(),
        }
    }

    /// Returns the packet length
    pub fn len(&self) -> usize {
        match self {
            IpPacket::V4(packet) => packet.packet().len(),
            IpPacket::V6(packet) => packet.packet().len(),
        }
    }
}

pub fn xlat_v6_to_v4(
    ipv6_packet: &Ipv6Packet,
    new_source: Ipv4Addr,
    new_dest: Ipv4Addr,
) -> Vec<u8> {
    let data = Ipv4 {
        version: 4,
        header_length: 20,
        dscp: 0,
        ecn: 0,
        total_length: 20 + ipv6_packet.payload().len() as u16,
        identification: 0,
        flags: 0,
        fragment_offset: 0,
        ttl: ipv6_packet.get_hop_limit(),
        next_level_protocol: ipv6_packet.get_next_header(),
        checksum: 0,
        source: new_source,
        destination: new_dest,
        options: vec![],
        payload: ipv6_packet.payload().to_vec(),
    };
    let mut buffer = vec![0; 20 + ipv6_packet.payload().len()];
    let mut packet = MutableIpv4Packet::new(buffer.as_mut()).unwrap();
    packet.populate(&data);
    packet.set_checksum(checksum(&packet.to_immutable()));
    let mut output = packet.to_immutable().packet().to_vec();
    // TODO: There is a bug here.. for now, force write header size
    output[0] = 0x45;
    output
}

pub fn xlat_v4_to_v6(
    ipv4_packet: &Ipv4Packet,
    new_source: Ipv6Addr,
    new_dest: Ipv6Addr,
) -> Vec<u8> {
    let data = Ipv6 {
        version: 6,
        traffic_class: 0,
        flow_label: 0,
        payload_length: 40 + ipv4_packet.payload().len() as u16,
        next_header: ipv4_packet.get_next_level_protocol(),
        hop_limit: ipv4_packet.get_ttl(),
        source: new_source,
        destination: new_dest,
        payload: ipv4_packet.payload().to_vec(),
    };
    let mut buffer = vec![0; 40 + ipv4_packet.payload().len()];
    let mut packet = MutableIpv6Packet::new(buffer.as_mut()).unwrap();
    packet.populate(&data);
    packet.to_immutable().packet().to_vec()
}
