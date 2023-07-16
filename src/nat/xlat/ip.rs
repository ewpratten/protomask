//! Translation logic for IPv4 and IPv6

use std::net::{Ipv4Addr, Ipv6Addr};

use pnet_packet::{ipv6::{Ipv6Packet, Ipv6, MutableIpv6Packet}, ipv4::{Ipv4, MutableIpv4Packet, self, Ipv4Packet}, Packet};

pub fn ipv6_to_ipv4(
    ipv6_packet: &Ipv6Packet,
    new_source: Ipv4Addr,
    new_dest: Ipv4Addr,
    decr_ttl: bool,
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
    let mut packet = MutableIpv4Packet::owned(vec![0; 20 + ipv6_packet.payload().len()]).unwrap();
    packet.populate(&data);
    packet.set_checksum(ipv4::checksum(&packet.to_immutable()));

    // Decrement the TTL if needed
    if decr_ttl {
        packet.set_ttl(packet.get_ttl() - 1);
    }

    let mut output = packet.to_immutable().packet().to_vec();
    // TODO: There is a bug here.. for now, force write header size
    output[0] = 0x45;
    output
}

pub fn ipv4_to_ipv6(
    ipv4_packet: &Ipv4Packet,
    new_source: Ipv6Addr,
    new_dest: Ipv6Addr,
    decr_ttl: bool,
) -> Vec<u8> {
    let data = Ipv6 {
        version: 6,
        traffic_class: 0,
        flow_label: 0,
        payload_length: ipv4_packet.payload().len() as u16,
        next_header: ipv4_packet.get_next_level_protocol(),
        hop_limit: ipv4_packet.get_ttl(),
        source: new_source,
        destination: new_dest,
        payload: ipv4_packet.payload().to_vec(),
    };
    let mut packet = MutableIpv6Packet::owned(vec![0; 40 + ipv4_packet.payload().len()]).unwrap();
    packet.populate(&data);

    // Decrement the TTL if needed
    if decr_ttl {
        packet.set_hop_limit(packet.get_hop_limit() - 1);
    }

    packet.to_immutable().packet().to_vec()
}
