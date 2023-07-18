use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::{Duration, Instant},
};

use bimap::BiHashMap;
use ipnet::{Ipv4Net, Ipv6Net};

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

    /// Check if an address is within the IPv4 pool
    pub fn is_address_within_pool(&self, address: &Ipv4Addr) -> bool {
        self.ipv4_pool.iter().any(|net| net.contains(address))
    }

    /// Calculate the translated version of any address
    pub fn calculate_xlat_addr(
        &mut self,
        input: &IpAddr,
        ipv6_nat64_prefix: &Ipv6Net,
    ) -> Result<IpAddr, TableError> {
        // Handle the incoming address type
        match input {
            // Handle IPv4
            IpAddr::V4(ipv4_addr) => {
                // If the address is in the IPv4 pool, it is a regular IPv4 address
                if self.is_address_within_pool(ipv4_addr) {
                    // This means we need to pass through to `get_reverse`
                    return Ok(IpAddr::V6(self.get_reverse(*ipv4_addr)?));
                }

                // Otherwise, it shall be embedded inside the ipv6 prefix
                let prefix_octets = ipv6_nat64_prefix.addr().octets();
                let address_octets = ipv4_addr.octets();
                return Ok(IpAddr::V6(Ipv6Addr::new(
                    u16::from_be_bytes([prefix_octets[0], prefix_octets[1]]),
                    u16::from_be_bytes([prefix_octets[2], prefix_octets[3]]),
                    u16::from_be_bytes([prefix_octets[4], prefix_octets[5]]),
                    u16::from_be_bytes([prefix_octets[6], prefix_octets[7]]),
                    u16::from_be_bytes([prefix_octets[8], prefix_octets[9]]),
                    u16::from_be_bytes([prefix_octets[10], prefix_octets[11]]),
                    u16::from_be_bytes([address_octets[0], address_octets[1]]),
                    u16::from_be_bytes([address_octets[2], address_octets[3]]),
                )));
            }

            // Handle IPv6
            IpAddr::V6(ipv6_addr) => {
                // If the address is in the IPv6 prefix, it is an embedded IPv4 address
                if ipv6_nat64_prefix.contains(ipv6_addr) {
                    let address_bytes = ipv6_addr.octets();
                    return Ok(IpAddr::V4(Ipv4Addr::new(
                        address_bytes[12],
                        address_bytes[13],
                        address_bytes[14],
                        address_bytes[15],
                    )));
                }

                // Otherwise, it is a regular IPv6 address and we can pass through to `get_or_assign_ipv4`
                return Ok(IpAddr::V4(self.get_or_assign_ipv4(*ipv6_addr)?));
            }
        }
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
                    let keep = now - *time < self.reservation_timeout;
                    if !keep {
                        log::info!("Pruned reservation: {} -> {}", v6, v4);
                    }
                    keep
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
