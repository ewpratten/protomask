//! Error types for this library

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid IPv6 prefix length: {0}. Must be one of 32, 40, 48, 56, 64, or 96")]
    InvalidPrefixLength(u8),
}