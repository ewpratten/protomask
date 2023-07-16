//! Packet type translation functionality

mod icmp;
mod ip;

pub use icmp::{icmpv6_to_icmp, icmp_to_icmpv6};
pub use ip::{ipv4_to_ipv6, ipv6_to_ipv4};