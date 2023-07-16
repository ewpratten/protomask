use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::{Duration, Instant},
};

use bimap::BiHashMap;
use ipnet::Ipv4Net;

/// Possible errors thrown in the address reservation process
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Address already reserved: {0}")]
    AddressAlreadyReserved(IpAddr),
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
    ) -> Result<(), Error> {
        // Check if either address is already reserved
        self.prune();
        if self.reservations.contains_left(&ipv6) {
            return Err(Error::AddressAlreadyReserved(ipv6.into()));
        } else if self.reservations.contains_right(&ipv4) {
            return Err(Error::AddressAlreadyReserved(ipv4.into()));
        }

        // Add the reservation
        self.reservations.insert(ipv6, ipv4);
        self.reservation_times.insert((ipv6, ipv4), None);
        Ok(())
    }

    /// Get or assign an IPv4 address for the given IPv6 address
    pub fn get_or_assign_ipv4(&mut self, ipv6: Ipv6Addr) -> Result<Ipv4Addr, Error> {
        // Prune old reservations
        self.prune();

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
            for ipv4 in ipv4_net.hosts(){
                // Check if this address is available for use
                if !self.reservations.contains_right(&ipv4) {
                    // Add the reservation
                    self.reservations.insert(ipv6, ipv4);
                    self.reservation_times
                        .insert((ipv6, ipv4), Some(Instant::now()));
                    return Ok(ipv4);
                }
            }
        }

        // If we get here, we failed to find an available address
        Err(Error::AddressPoolDepleted)
    }

    /// Try to find an IPv6 address for the given IPv4 address
    pub fn get_reverse(&mut self, ipv4: Ipv4Addr) -> Option<Ipv6Addr> {
        // Prune old reservations
        self.prune();

        // If the IPv4 address is already reserved, return the IPv6 address
        if let Some(ipv6) = self.reservations.get_by_right(&ipv4) {
            // Update the reservation time
            self.reservation_times
                .insert((*ipv6, ipv4), Some(Instant::now()));

            // Return the v6 address
            return Some(*ipv6);
        }

        // Otherwise, there is no matching reservation
        None
    }
}

impl Nat64Table {
    /// Prune old reservations
    pub fn prune(&mut self) {
        let now = Instant::now();

        // Prune from the reservation map
        self.reservations.retain(|v6, v4| {
            if let Some(time) = self.reservation_times.get(&(*v6, *v4)) {
                if let Some(time) = time {
                    now - *time < self.reservation_timeout
                } else {
                    true
                }
            } else {
                true
            }
        });

        // Remove all times assigned to reservations that no longer exist
        self.reservation_times.retain(|(v6, v4), _| {
            self.reservations.contains_left(v6) && self.reservations.contains_right(v4)
        });
    }
}
