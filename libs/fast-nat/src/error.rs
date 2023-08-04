#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ipv4 address does not belong to the NAT pool: {0:02x}")]
    InvalidIpv4Address(u32),
    #[error("IPv4 pool exhausted. All {0} spots filled")]
    Ipv4PoolExhausted(usize),
}
