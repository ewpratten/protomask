use std::net::IpAddr;

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("Mismatched source and destination address family: source={0:?}, destination={1:?}")]
    MismatchedAddressFamily(IpAddr, IpAddr),
    #[error("Packet too short: {0}")]
    TooShort(usize, Vec<u8>),
    #[error("Unsupported ICMP type: {0}")]
    UnsupportedIcmpType(u8),
    #[error("Unsupported ICMPv6 type: {0}")]
    UnsupportedIcmpv6Type(u8),
}

