#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod bimap;
mod cpnat;
pub mod error;
mod nat;
mod timeout;

pub use cpnat::{CrossProtocolNetworkAddressTable, CrossProtocolNetworkAddressTableWithIpv4Pool};
pub use nat::NetworkAddressTable;
