#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_safety_doc)]

mod error;
mod embed;
mod extract;
pub use embed::{embed_ipv4_addr, embed_ipv4_addr_unchecked};
pub use error::Error;
pub use extract::{extract_ipv4_addr, extract_ipv4_addr_unchecked};

/// All allowed IPv6 prefix lengths according to [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
/// 
/// While any prefix length between 32 and 96 bits can in theory work with this library, 
/// the RFC strictly defines a list of allowed IPv6 prefix to be used for embedding IPv4 addresses. They are:
/// - 32 bits
/// - 40 bits
/// - 48 bits
/// - 56 bits
/// - 64 bits
/// - 96 bits
pub const ALLOWED_PREFIX_LENS: [u8; 6] = [32, 40, 48, 56, 64, 96];
