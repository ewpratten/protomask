/// All possible errors thrown by `interproto` functions
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("Packet too short. Expected at least {expected} bytes, got {actual}")]
    PacketTooShort { expected: usize, actual: usize },
    #[error("Unsupported ICMP type: {0}")]
    UnsupportedIcmpType(u8),
    #[error("Unsupported ICMPv6 type: {0}")]
    UnsupportedIcmpv6Type(u8),
}

/// Result type for `interproto`
pub type Result<T> = std::result::Result<T, Error>;
