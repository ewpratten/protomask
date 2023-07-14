use std::net::{Ipv4Addr, Ipv6Addr};

use ipnet::Ipv4Net;

/// Represents a pair of IP addresses for a dual-stack host or mapping
#[derive(Debug, serde::Deserialize)]
pub struct AddressPair {
    /// IPv4 address
    pub v4: Ipv4Addr,
    /// IPv6 address
    pub v6: Ipv6Addr,
}

// /// Represents a pool of IPv4 addresses
// #[derive(Debug, serde::Deserialize)]
// pub struct Ipv4Pool {
//     /// All possible addresses
//     pub prefixes: Vec<Ipv4Net>,
//     /// Addresses that cannot be dynamically assigned
//     pub reservations: Vec<Ipv4Addr>,
// }

// impl Ipv4Pool {
//     /// Construct a new `Ipv4Pool`
//     pub fn new(prefixes: Vec<Ipv4Net>) -> Self {
//         Self {
//             prefixes,
//             reservations: Vec::new(),
//         }
//     }

//     /// Reserve 
// }