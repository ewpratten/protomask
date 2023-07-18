use pnet_packet::{
    icmp::{IcmpCode, IcmpType},
    Packet,
};

use crate::packet::error::PacketError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IcmpPacket<T> {
    pub icmp_type: IcmpType,
    pub icmp_code: IcmpCode,
    pub payload: T,
}

impl<T> IcmpPacket<T> {
    /// Construct a new ICMPv6 packet
    pub fn new(icmp_type: IcmpType, icmp_code: IcmpCode, payload: T) -> Self {
        Self {
            icmp_type,
            icmp_code,
            payload,
        }
    }
}

impl<T> TryFrom<Vec<u8>> for IcmpPacket<T>
where
    T: TryFrom<Vec<u8>, Error = PacketError>,
{
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        // Parse the packet
        let packet =
            pnet_packet::icmp::IcmpPacket::new(&bytes).ok_or(PacketError::TooShort(bytes.len()))?;

        // Return the packet
        Ok(Self {
            icmp_type: packet.get_icmp_type(),
            icmp_code: packet.get_icmp_code(),
            payload: packet.payload().to_vec().try_into()?,
        })
    }
}

impl<T> Into<Vec<u8>> for IcmpPacket<T>
where
    T: Into<Vec<u8>>,
{
    fn into(self) -> Vec<u8> {
        // Convert the payload into raw bytes
        let payload: Vec<u8> = self.payload.into();

        // Allocate a mutable packet to write into
        let total_length =
            pnet_packet::icmp::MutableIcmpPacket::minimum_packet_size() + payload.len();
        let mut output =
            pnet_packet::icmp::MutableIcmpPacket::owned(vec![0u8; total_length]).unwrap();

        // Write the type and code
        output.set_icmp_type(self.icmp_type);
        output.set_icmp_code(self.icmp_code);

        // Write the payload
        output.set_payload(&payload);

        // Calculate the checksum
        output.set_checksum(0);
        output.set_checksum(pnet_packet::icmp::checksum(&output.to_immutable()));

        // Return the raw bytes
        output.packet().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use pnet_packet::icmp::IcmpTypes;

    use super::*;

    // Test packet construction
    #[test]
    #[rustfmt::skip]
    fn test_packet_construction() {
        // Make a new packet
        let packet = IcmpPacket::new(
            IcmpTypes::EchoRequest,
            IcmpCode(0),
            "Hello, world!".as_bytes().to_vec(),
        );

        // Convert to raw bytes
        let packet_bytes: Vec<u8> = packet.into();

        // Check the contents
        assert!(packet_bytes.len() >= 4 + 13);
        assert_eq!(packet_bytes[0], IcmpTypes::EchoRequest.0);
        assert_eq!(packet_bytes[1], 0);
        assert_eq!(u16::from_be_bytes([packet_bytes[2], packet_bytes[3]]), 0xb6b3);
        assert_eq!(
            &packet_bytes[4..],
            "Hello, world!".as_bytes().to_vec().as_slice()
        );
    }
}
