#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod error;

mod embed;
mod extract;
pub use embed::embed_ipv4_addr;

/// All allowed IPv6 prefix lengths according to [RFC6052 Section 2.2](https://datatracker.ietf.org/doc/html/rfc6052#section-2.2)
pub const ALLOWED_PREFIX_LENS: [u8; 6] = [32, 40, 48, 56, 64, 96];
