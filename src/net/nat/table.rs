
// use rustc_hash::FxHashMap;

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct IpBimap {
//     v4_to_v6: FxHashMap<u32, u128>,
//     v6_to_v4: FxHashMap<u128, u32>,
// }

// impl IpBimap {
//     /// Construct a new `IpBimap`
//     pub fn new() -> Self {        
//         Self {
//             v4_to_v6: FxHashMap::default(),
//             v6_to_v4: FxHashMap::default(),
//         }
//     }

//     /// Insert a new mapping
//     pub fn insert(&mut self, v4: u32, v6: u128) {
//         self.v4_to_v6.insert(v4, v6);
//         self.v6_to_v4.insert(v6, v4);
//     }

//     /// Remove a mapping
//     pub fn remove(&mut self, v4: u32, v6: u128) {
//         self.v4_to_v6.remove(&v4);
//         self.v6_to_v4.remove(&v6);
//     }

//     /// Get the IPv6 address for a given IPv4 address
//     pub fn get_v6(&self, v4: u32) -> Option<u128> {
//         self.v4_to_v6.get(&v4).copied()
//     }

//     /// Get the IPv4 address for a given IPv6 address
//     pub fn get_v4(&self, v6: u128) -> Option<u32> {
//         self.v6_to_v4.get(&v6).copied()
//     }
// }


use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::{Duration, Instant},
};

use bimap::BiHashMap;
use ipnet::Ipv4Net;

use crate::utils::metrics::{IPV4_POOL_SIZE, IPV4_POOL_RESERVED};


/// Possible errors thrown in the address reservation process
#[derive(Debug, thiserror::Error)]
pub enum TableError {
    #[error("Address already reserved: {0}")]
    AddressAlreadyReserved(IpAddr),
    #[error("IPv4 address has no IPv6 mapping: {0}")]
    NoIpv6Mapping(Ipv4Addr),
    #[error("Address pool depleted")]
    AddressPoolDepleted,
}

/// A NAT address table
#[derive(Debug)]
pub struct Nat64Table {
    /// All possible IPv4 addresses that can be used
    ipv4_pool: Vec<Ipv4Net>,
    /// Current reservations
    reservations: BiHashMap<Ipv6Addr, Ipv4Addr>,
    /// The timestamp of each reservation (used for pruning)
    reservation_times: HashMap<(Ipv6Addr, Ipv4Addr), Option<Instant>>,
    /// The maximum amount of time to reserve an address pair for
    reservation_timeout: Duration,
}

impl Nat64Table {
    /// Construct a new NAT64 table
    ///
    /// **Arguments:**
    /// - `ipv4_pool`: The pool of IPv4 addresses to use in the mapping process
    /// - `reservation_timeout`: The amount of time to reserve an address pair for
    pub fn new(ipv4_pool: Vec<Ipv4Net>, reservation_timeout: Duration) -> Self {
        // Track the total pool size
        let total_size: usize = ipv4_pool.iter().map(|net| net.hosts().count()).sum();
        IPV4_POOL_SIZE.set(total_size as i64);

        Self {
            ipv4_pool,
            reservations: BiHashMap::new(),
            reservation_times: HashMap::new(),
            reservation_timeout,
        }
    }

    /// Make a reservation for an IP address pair for eternity
    pub fn add_infinite_reservation(
        &mut self,
        ipv6: Ipv6Addr,
        ipv4: Ipv4Addr,
    ) -> Result<(), TableError> {
        // Check if either address is already reserved
        self.prune();
        self.track_utilization();
        if self.reservations.contains_left(&ipv6) {
            return Err(TableError::AddressAlreadyReserved(ipv6.into()));
        } else if self.reservations.contains_right(&ipv4) {
            return Err(TableError::AddressAlreadyReserved(ipv4.into()));
        }

        // Add the reservation
        self.reservations.insert(ipv6, ipv4);
        self.reservation_times.insert((ipv6, ipv4), None);
        log::info!("Added infinite reservation: {} -> {}", ipv6, ipv4);
        Ok(())
    }

    /// Check if a given address exists in the table
    pub fn contains(&self, address: &IpAddr) -> bool {
        match address {
            IpAddr::V4(ipv4) => self.reservations.contains_right(ipv4),
            IpAddr::V6(ipv6) => self.reservations.contains_left(ipv6),
        }
    }

    /// Get or assign an IPv4 address for the given IPv6 address
    pub fn get_or_assign_ipv4(&mut self, ipv6: Ipv6Addr) -> Result<Ipv4Addr, TableError> {
        // Prune old reservations
        self.prune();
        self.track_utilization();

        // If the IPv6 address is already reserved, return the IPv4 address
        if let Some(ipv4) = self.reservations.get_by_left(&ipv6) {
            // Update the reservation time
            self.reservation_times
                .insert((ipv6, *ipv4), Some(Instant::now()));

            // Return the v4 address
            return Ok(*ipv4);
        }

        // Otherwise, try to assign a new IPv4 address
        for ipv4_net in &self.ipv4_pool {
            for ipv4 in ipv4_net.hosts() {
                // Check if this address is available for use
                if !self.reservations.contains_right(&ipv4) {
                    // Add the reservation
                    self.reservations.insert(ipv6, ipv4);
                    self.reservation_times
                        .insert((ipv6, ipv4), Some(Instant::now()));
                    log::info!("Assigned new reservation: {} -> {}", ipv6, ipv4);
                    return Ok(ipv4);
                }
            }
        }

        // If we get here, we failed to find an available address
        Err(TableError::AddressPoolDepleted)
    }

    /// Try to find an IPv6 address for the given IPv4 address
    pub fn get_reverse(&mut self, ipv4: Ipv4Addr) -> Result<Ipv6Addr, TableError> {
        // Prune old reservations
        self.prune();
        self.track_utilization();

        // If the IPv4 address is already reserved, return the IPv6 address
        if let Some(ipv6) = self.reservations.get_by_right(&ipv4) {
            // Update the reservation time
            self.reservation_times
                .insert((*ipv6, ipv4), Some(Instant::now()));

            // Return the v6 address
            return Ok(*ipv6);
        }

        // Otherwise, there is no matching reservation
        Err(TableError::NoIpv6Mapping(ipv4))
    }
}

impl Nat64Table {
    /// Prune old reservations
    fn prune(&mut self) {
        let now = Instant::now();

        // Prune from the reservation map
        self.reservations.retain(|v6, v4| {
            if let Some(Some(time)) = self.reservation_times.get(&(*v6, *v4)) {
                let keep = now - *time < self.reservation_timeout;
                if !keep {
                    log::info!("Pruned reservation: {} -> {}", v6, v4);
                }
                keep
            } else {
                true
            }
        });

        // Remove all times assigned to reservations that no longer exist
        self.reservation_times.retain(|(v6, v4), _| {
            self.reservations.contains_left(v6) && self.reservations.contains_right(v4)
        });
    }

    fn track_utilization(&self) {
        // Count static and dynamic in a single pass
        let (total_dynamic_reservations, total_static_reservations) = self
            .reservation_times
            .iter()
            .map(|((_v6, _v4), time)| match time {
                Some(_) => (1, 0),
                None => (0, 1),
            })
            .fold((0, 0), |(a1, a2), (b1, b2)| (a1 + b1, a2 + b2));

        // Track the values
        IPV4_POOL_RESERVED
            .with_label_values(&["dynamic"])
            .set(i64::from(total_dynamic_reservations));
        IPV4_POOL_RESERVED
            .with_label_values(&["static"])
            .set(i64::from(total_static_reservations));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_infinite_reservation() {
        let mut table = Nat64Table::new(
            vec![Ipv4Net::new(Ipv4Addr::new(192, 0, 2, 0), 24).unwrap()],
            Duration::from_secs(60),
        );

        // Add a reservation
        table
            .add_infinite_reservation("2001:db8::1".parse().unwrap(), "192.0.2.1".parse().unwrap())
            .unwrap();

        // Check that it worked
        assert_eq!(
            table
                .reservations
                .get_by_left(&"2001:db8::1".parse().unwrap()),
            Some(&"192.0.2.1".parse().unwrap())
        );
    }
}
