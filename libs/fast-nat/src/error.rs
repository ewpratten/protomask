use std::net::Ipv4Addr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ipv4 address does not belong to the NAT pool: {0}")]
    InvalidIpv4Address(Ipv4Addr),
    #[error("IPv4 pool exhausted")]
    Ipv4PoolExhausted,
}
