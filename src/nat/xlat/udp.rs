// use super::PacketTranslationError;
// use pnet_packet::{
//     udp::{self, MutableUdpPacket, UdpPacket},
//     Packet,
// };
// use std::net::{Ipv4Addr, Ipv6Addr};

// /// Translate an IPv4 UDP packet into an IPv6 UDP packet (aka: recalculate checksum)
// pub fn translate_udp_4_to_6(
//     ipv4_udp: UdpPacket,
//     new_source: Ipv6Addr,
//     new_dest: Ipv6Addr,
// ) -> Result<UdpPacket, PacketTranslationError> {
//     // Create a mutable clone of the IPv4 UDP packet, so it can be adapted for use in IPv6
//     let mut ipv6_udp = MutableUdpPacket::owned(ipv4_udp.packet().to_vec())
//         .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv4_udp.packet().len()))?;

//     // Rewrite the checksum for use in an IPv6 packet
//     ipv6_udp.set_checksum(0);
//     ipv6_udp.set_checksum(udp::ipv6_checksum(
//         &ipv4_udp.to_immutable(),
//         &new_source,
//         &new_dest,
//     ));

//     // Return the translated packet
//     Ok(UdpPacket::owned(ipv6_udp.packet().to_vec()).unwrap())
// }

// /// Translate an IPv6 UDP packet into an IPv4 UDP packet (aka: recalculate checksum)
// pub fn translate_udp_6_to_4(
//     ipv6_udp: UdpPacket,
//     new_source: Ipv4Addr,
//     new_dest: Ipv4Addr,
// ) -> Result<UdpPacket, PacketTranslationError> {
//     // Create a mutable clone of the IPv6 UDP packet, so it can be adapted for use in IPv4
//     let mut ipv4_udp = MutableUdpPacket::owned(ipv6_udp.packet().to_vec())
//         .ok_or_else(|| PacketTranslationError::InputPacketTooShort(ipv6_udp.packet().len()))?;

//     // Rewrite the checksum for use in an IPv4 packet
//     ipv4_udp.set_checksum(0);
//     ipv4_udp.set_checksum(udp::ipv4_checksum(
//         &ipv6_udp.to_immutable(),
//         &new_source,
//         &new_dest,
//     ));

//     // Return the translated packet
//     Ok(UdpPacket::owned(ipv4_udp.packet().to_vec()).unwrap())
// }

// #[cfg(test)]
// mod tests {
//     use crate::into_udp;

//     use super::*;

//     #[test]
//     fn test_udp_4_to_6() {
//         // Build an example UDP packet
//         let input = into_udp!(vec![
//             0, 255, // Source port
//             0, 128, // Destination port
//             0, 4, // Length
//             0, 0, // Checksum (doesn't matter)
//             1, 2, 3, 4 // Data
//         ])
//         .unwrap();

//         // Translate to IPv6
//         let output = translate_udp_4_to_6(
//             input,
//             "2001:db8::1".parse().unwrap(),
//             "2001:db8::2".parse().unwrap(),
//         );

//         // Check the output
//         assert!(output.is_ok());
//         let output = output.unwrap();

//         // Check the output's contents
//         assert_eq!(output.get_source(), 255);
//         assert_eq!(output.get_destination(), 128);
//         assert_eq!(output.get_length(), 4);
//         assert_eq!(output.payload(), &[1, 2, 3, 4]);
//     }

//     #[test]
//     fn test_udp_6_to_4() {
//         // Build an example UDP packet
//         let input = into_udp!(vec![
//             0, 255, // Source port
//             0, 128, // Destination port
//             0, 4, // Length
//             0, 0, // Checksum (doesn't matter)
//             1, 2, 3, 4 // Data
//         ])
//         .unwrap();

//         // Translate to IPv4
//         let output = translate_udp_6_to_4(
//             input,
//             "192.0.2.1".parse().unwrap(),
//             "192.0.2.2".parse().unwrap(),
//         );

//         // Check the output
//         assert!(output.is_ok());
//         let output = output.unwrap();

//         // Check the output's contents
//         assert_eq!(output.get_source(), 255);
//         assert_eq!(output.get_destination(), 128);
//         assert_eq!(output.get_length(), 4);
//         assert_eq!(output.payload(), &[1, 2, 3, 4]);
//     }
// }
