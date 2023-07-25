//! The NAT table
//!
//! ## Internals
//!
//! The NAT table is responsible for tracking which IPv6 addresses are
//! mapped to which IPv4 addresses (and vice versa).
//!
//! When a packet is received from an IPv6 host destined for an IPv4
//! host, we don't want to randomly assign a new source address.
//! Hosts on each end generally expect a stable "neighbor" to talk to.
//!
//! The NAT table solves this by storing a bi-directional map of IP
//! addresses in the form of:
//! ```text
//! (ipv6 <-> ipv4)
//! ```
//!
//! Since its possible for a malicious IPv6 user to use a `/64` to
//! spam us with packets (depleting the ipv4 pool), we also need to
//! enforce a maximum "hold time" for each address mapping. This way,
//! any IPv6 host that hasn't talked for `n` seconds will free up its
//! IPv4 address for another IPv6 host to possibly use.
//!
//! While this isn't the best solution, its fairly OK for now.
//!
//! In order to keep track of the hold time for a mapping, we use a second map:
//! ```text
//! ((ipv6, ipv4) -> (last_packet_time, Option<timeout_duration>))
//! ```
//!
//! *(Note, some mappings are "static" and will never timeout)*
//!
//! ## Serialization
//!
//! Users might want their mappings to persist across restarts of `protomask`.
//! This means that sessions *probably* won't be broken during a version upgrade,
//! server restart, or config tweak.
//!
//! To achieve this, we need to serialize the NAT table to disk.
//!
//! Serialized data is stored in the form:
//! ```text
//! (ipv6, ipv4, Option<timeout_duration>)
//! ```
//!
//! Upon loading the program again, this data is re-loaded into the
//! existing data structures. **NOTE:** We don't store the last packet
//! time for the sake of simplicity. All mappings will be assumed to
//! be fresh on restart (giving another `n` seconds of time to each one).

use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
    path::Path,
    time::{Duration, Instant},
};

use bimap::BiMap;
use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};

use crate::metrics::IPV4_POOL_RESERVED;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializedReservation {
    ipv6: Ipv6Addr,
    ipv4: Ipv4Addr,
    duration: Option<Duration>,
}

#[derive(Debug, thiserror::Error)]
pub enum Nat64TableError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ReaderError(#[from] flexbuffers::ReaderError),
    #[error(transparent)]
    DeserializationError(#[from] flexbuffers::DeserializationError),
    #[error(transparent)]
    SerializationError(#[from] flexbuffers::SerializationError),
}

#[derive(Debug)]
pub struct Nat64Table {
    /// All available IPv4 addresses
    ipv4_pool: Vec<Ipv4Net>,
    /// All current address mappings
    mappings: BiMap<Ipv6Addr, Ipv4Addr>,
    /// The hold timers for each mapping
    hold_timers: HashMap<(Ipv6Addr, Ipv4Addr), (Instant, Option<Duration>)>,
}

impl Nat64Table {
    /// Create a new `Nat64Table` instance
    pub fn new<P: AsRef<Path>>(
        ipv4_pool: Vec<Ipv4Net>,
        state_file: Option<P>,
    ) -> Result<Self, Nat64TableError> {
        // Allocate a new table for mappings and timers
        let mut mapping_table = BiMap::new();
        let mut hold_timers = HashMap::new();

        // Keep track of "now" for the purposes of initialization
        let now = Instant::now();

        // If we have a file to read
        if let Some(state_file) = state_file {
            // Try to parse it
            let bytes = std::fs::read(state_file)?;
            let deserializer = flexbuffers::Reader::get_root(bytes.as_slice())?;
            let on_disk_reservations = Vec::<SerializedReservation>::deserialize(deserializer)?;

            // Write every reservation to the tables created above (ignoring any reservation that is outside of the pool)
            for reservation in &on_disk_reservations {
                if ipv4_pool.iter().any(|net| net.contains(&reservation.ipv4)) {
                    log::debug!(
                        "Loaded reservation from disk: {} -> {} ({})",
                        reservation.ipv6,
                        reservation.ipv4,
                        match reservation.duration {
                            Some(duration) => format!("{}s", duration.as_secs()),
                            None => "infinite".to_string(),
                        }
                    );
                    mapping_table.insert(reservation.ipv6, reservation.ipv4);
                    hold_timers.insert(
                        (reservation.ipv6, reservation.ipv4),
                        (now, reservation.duration),
                    );

                    // Update prometheus counters to reflect the new reservation
                    IPV4_POOL_RESERVED
                        .with_label_values(match reservation.duration {
                            Some(_) => &["dynamic"],
                            None => &["static"],
                        })
                        .inc();
                }
            }
        }

        Ok(Self {
            ipv4_pool,
            mappings: mapping_table,
            hold_timers,
        })
    }

    /// Tracks a new IP mapping
    pub fn add(&mut self, ipv6: Ipv6Addr, ipv4: Ipv4Addr, timeout: Option<Duration>) {
        // Remove any old mappings
        self.hold_timers
            .iter()
            .filter(|((v6, v4), (time, duration))| {
                if let Some(duration) = duration {
                    *v6 == ipv6 && *v4 == ipv4 && time.elapsed() > *duration
                } else {
                    false
                }
            })
            .for_each(|((v6, v4), (_, duration))| {
                log::debug!("Removed old mapping: {} -> {}", v6, v4);
                self.mappings.remove_by_left(v6);
                self.mappings.remove_by_right(v4);
                self.hold_timers.remove(&(*v6, *v4));

                // Update the prometheus counter
                IPV4_POOL_RESERVED
                    .with_label_values(match duration {
                        Some(_) => &["dynamic"],
                        None => &["static"],
                    })
                    .dec();
            });

        // Add the mapping if it doesn't already exist
        if !(self.mappings.contains_left(&ipv6) || self.mappings.contains_right(&ipv4)) {
            self.mappings.insert(ipv6, ipv4);

            // Update the prometheus counter
            IPV4_POOL_RESERVED
                .with_label_values(match timeout {
                    Some(_) => &["dynamic"],
                    None => &["static"],
                })
                .inc();
        }

        // Update the hold timer
        self.hold_timers
            .insert((ipv6, ipv4), (Instant::now(), timeout));
    }

    /// Save the whole table to a file for later re-loading
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Nat64TableError> {
        // Build a serializer
        let mut serializer = flexbuffers::FlexbufferSerializer::new();

        // Build a list of reservations to serialize
        let mut reservations = Vec::new();
        for (ipv6, ipv4) in &self.mappings {
            let duration = self
                .hold_timers
                .get(&(*ipv6, *ipv4))
                .map(|(_, duration)| *duration)
                .unwrap();
            reservations.push(SerializedReservation {
                ipv6: *ipv6,
                ipv4: *ipv4,
                duration,
            });
        }

        // Serialize the data
        reservations.serialize(&mut serializer)?;

        // Write to disk
        std::fs::write(path, serializer.view())?;

        Ok(())
    }
}

// use std::{net::{Ipv6Addr, Ipv4Addr}, time::Duration};

// /// Represents an amount of time. Either infinite or finite.
// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub enum ReservationDuration {
//     Infinite,
//     Finite(Duration),
// }

// /// Represents the data stored on disk when persisting the NAT table
// /// NOTE: The duration value is *not* stored because it will be re-initialized on startup
// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// struct SerializedReservation {
//     ipv6: Ipv6Addr,
//     ipv4: Ipv4Addr,
//     infinite: bool
// }

// /// The NAT table
// #[derive(Debug)]
// pub struct Nat64Table {
//     /// All possible IPv4 addresses that can be used
//     ipv4_pool: Vec<Ipv4Addr>,
//     /// All current address mappings
//     reservations: Vec<(Ipv6Addr, Ipv4Addr)>,
//     /// The timeouts for each reservation
//     reservation_timeouts: Vec<ReservationDuration>,
// }

// use std::{
//     collections::HashMap,
//     net::{IpAddr, Ipv4Addr, Ipv6Addr},
//     time::{Duration, Instant},
// };

// use bimap::BiHashMap;
// use ipnet::Ipv4Net;

// use crate::metrics::{IPV4_POOL_RESERVED, IPV4_POOL_SIZE};

// /// Possible errors thrown in the address reservation process
// #[derive(Debug, thiserror::Error)]
// pub enum TableError {
//     #[error("Address already reserved: {0}")]
//     AddressAlreadyReserved(IpAddr),
//     #[error("IPv4 address has no IPv6 mapping: {0}")]
//     NoIpv6Mapping(Ipv4Addr),
//     #[error("Address pool depleted")]
//     AddressPoolDepleted,
// }

// /// A NAT address table
// #[derive(Debug)]
// pub struct Nat64Table {
//     /// All possible IPv4 addresses that can be used
//     ipv4_pool: Vec<Ipv4Net>,
//     /// Current reservations
//     reservations: BiHashMap<Ipv6Addr, Ipv4Addr>,
//     /// The timestamp of each reservation (used for pruning)
//     reservation_times: HashMap<(Ipv6Addr, Ipv4Addr), Option<Instant>>,
//     /// The maximum amount of time to reserve an address pair for
//     reservation_timeout: Duration,
// }

// impl Nat64Table {
//     /// Construct a new NAT64 table
//     ///
//     /// **Arguments:**
//     /// - `ipv4_pool`: The pool of IPv4 addresses to use in the mapping process
//     /// - `reservation_timeout`: The amount of time to reserve an address pair for
//     pub fn new(ipv4_pool: Vec<Ipv4Net>, reservation_timeout: Duration) -> Self {
//         // Track the total pool size
//         let total_size: usize = ipv4_pool.iter().map(|net| net.hosts().count()).sum();
//         IPV4_POOL_SIZE.set(total_size as i64);

//         Self {
//             ipv4_pool,
//             reservations: BiHashMap::new(),
//             reservation_times: HashMap::new(),
//             reservation_timeout,
//         }
//     }

//     /// Make a reservation for an IP address pair for eternity
//     pub fn add_infinite_reservation(
//         &mut self,
//         ipv6: Ipv6Addr,
//         ipv4: Ipv4Addr,
//     ) -> Result<(), TableError> {
//         // Check if either address is already reserved
//         self.prune();
//         self.track_utilization();
//         if self.reservations.contains_left(&ipv6) {
//             return Err(TableError::AddressAlreadyReserved(ipv6.into()));
//         } else if self.reservations.contains_right(&ipv4) {
//             return Err(TableError::AddressAlreadyReserved(ipv4.into()));
//         }

//         // Add the reservation
//         self.reservations.insert(ipv6, ipv4);
//         self.reservation_times.insert((ipv6, ipv4), None);
//         log::info!("Added infinite reservation: {} -> {}", ipv6, ipv4);
//         Ok(())
//     }

//     /// Check if a given address exists in the table
//     pub fn contains(&self, address: &IpAddr) -> bool {
//         match address {
//             IpAddr::V4(ipv4) => self.reservations.contains_right(ipv4),
//             IpAddr::V6(ipv6) => self.reservations.contains_left(ipv6),
//         }
//     }

//     /// Get or assign an IPv4 address for the given IPv6 address
//     pub fn get_or_assign_ipv4(&mut self, ipv6: Ipv6Addr) -> Result<Ipv4Addr, TableError> {
//         // Prune old reservations
//         self.prune();
//         self.track_utilization();

//         // If the IPv6 address is already reserved, return the IPv4 address
//         if let Some(ipv4) = self.reservations.get_by_left(&ipv6) {
//             // Update the reservation time
//             self.reservation_times
//                 .insert((ipv6, *ipv4), Some(Instant::now()));

//             // Return the v4 address
//             return Ok(*ipv4);
//         }

//         // Otherwise, try to assign a new IPv4 address
//         for ipv4_net in &self.ipv4_pool {
//             for ipv4 in ipv4_net.hosts() {
//                 // Check if this address is available for use
//                 if !self.reservations.contains_right(&ipv4) {
//                     // Add the reservation
//                     self.reservations.insert(ipv6, ipv4);
//                     self.reservation_times
//                         .insert((ipv6, ipv4), Some(Instant::now()));
//                     log::info!("Assigned new reservation: {} -> {}", ipv6, ipv4);
//                     return Ok(ipv4);
//                 }
//             }
//         }

//         // If we get here, we failed to find an available address
//         Err(TableError::AddressPoolDepleted)
//     }

//     /// Try to find an IPv6 address for the given IPv4 address
//     pub fn get_reverse(&mut self, ipv4: Ipv4Addr) -> Result<Ipv6Addr, TableError> {
//         // Prune old reservations
//         self.prune();
//         self.track_utilization();

//         // If the IPv4 address is already reserved, return the IPv6 address
//         if let Some(ipv6) = self.reservations.get_by_right(&ipv4) {
//             // Update the reservation time
//             self.reservation_times
//                 .insert((*ipv6, ipv4), Some(Instant::now()));

//             // Return the v6 address
//             return Ok(*ipv6);
//         }

//         // Otherwise, there is no matching reservation
//         Err(TableError::NoIpv6Mapping(ipv4))
//     }
// }

// impl Nat64Table {

//     // fn add(&mut self, ipv6: Ipv6Addr, ipv4: Ipv4Addr, )

//     /// Prune old reservations
//     fn prune(&mut self) {
//         let now = Instant::now();

//         // Prune from the reservation map
//         self.reservations.retain(|v6, v4| {
//             if let Some(Some(time)) = self.reservation_times.get(&(*v6, *v4)) {
//                 let keep = now - *time < self.reservation_timeout;
//                 if !keep {
//                     log::info!("Pruned reservation: {} -> {}", v6, v4);
//                 }
//                 keep
//             } else {
//                 true
//             }
//         });

//         // Remove all times assigned to reservations that no longer exist
//         self.reservation_times.retain(|(v6, v4), _| {
//             self.reservations.contains_left(v6) && self.reservations.contains_right(v4)
//         });
//     }

// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_add_infinite_reservation() {
//         let mut table = Nat64Table::new(
//             vec![Ipv4Net::new(Ipv4Addr::new(192, 0, 2, 0), 24).unwrap()],
//             Duration::from_secs(60),
//         );

//         // Add a reservation
//         table
//             .add_infinite_reservation("2001:db8::1".parse().unwrap(), "192.0.2.1".parse().unwrap())
//             .unwrap();

//         // Check that it worked
//         assert_eq!(
//             table
//                 .reservations
//                 .get_by_left(&"2001:db8::1".parse().unwrap()),
//             Some(&"192.0.2.1".parse().unwrap())
//         );
//     }
// }
