use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use ipnet::Ipv4Net;
use rustc_hash::FxHashMap;

use crate::{bimap::BiHashMap, error::Error, timeout::MaybeTimeout};

/// A table of network address mappings across IPv4 and IPv6
#[derive(Debug)]
pub struct CrossProtocolNetworkAddressTable {
    /// Internal address map
    addr_map: BiHashMap<u32, u128>,
    /// Secondary map used to keep track of timeouts
    timeouts: FxHashMap<(u32, u128), MaybeTimeout>,
}

impl CrossProtocolNetworkAddressTable {
    /// Construct a new empty `CrossProtocolNetworkAddressTable`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Prune all old mappings
    pub fn prune(&mut self) {
        log::trace!("Pruning old network address mappings");

        // Compare all mappings against a common timestamp
        let now = std::time::Instant::now();

        // Remove all old mappings from both the bimap and the timeouts map
        self.timeouts.retain(|(left, right), timeout| {
            match timeout {
                // Retain all indefinite mappings
                MaybeTimeout::Never => true,
                // Only retain mappings that haven't timed out yet
                MaybeTimeout::After { duration, start } => {
                    let should_retain = now.duration_since(*start) < *duration;
                    if !should_retain {
                        log::trace!(
                            "Mapping {:?} -> {:?} has timed out and will be removed",
                            left,
                            right
                        );
                        self.addr_map.remove(left, right);
                    }
                    should_retain
                }
            }
        });
    }

    /// Insert a new indefinite mapping
    pub fn insert_indefinite(&mut self, ipv4: Ipv4Addr, ipv6: Ipv6Addr) {
        self.prune();
        let (ipv4, ipv6) = (ipv4.into(), ipv6.into());
        self.addr_map.insert(ipv4, ipv6);
        self.timeouts.insert((ipv4, ipv6), MaybeTimeout::Never);
    }

    /// Insert a new mapping with a finite time-to-live
    pub fn insert(&mut self, ipv4: Ipv4Addr, ipv6: Ipv6Addr, duration: Duration) {
        self.prune();
        let (ipv4, ipv6) = (ipv4.into(), ipv6.into());
        self.addr_map.insert(ipv4, ipv6);
        self.timeouts.insert(
            (ipv4, ipv6),
            MaybeTimeout::After {
                duration,
                start: std::time::Instant::now(),
            },
        );
    }

    /// Get the IPv6 address for a given IPv4 address
    #[must_use]
    pub fn get_ipv6(&self, ipv4: &Ipv4Addr) -> Option<Ipv6Addr> {
        self.addr_map
            .get_right(&(*ipv4).into())
            .map(|addr| (*addr).into())
    }

    /// Get the IPv4 address for a given IPv6 address
    #[must_use]
    pub fn get_ipv4(&self, ipv6: &Ipv6Addr) -> Option<Ipv4Addr> {
        self.addr_map
            .get_left(&(*ipv6).into())
            .map(|addr| (*addr).into())
    }

    /// Get the number of mappings in the table
    #[must_use]
    pub fn len(&self) -> usize {
        self.addr_map.len()
    }

    /// Check if the table is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.addr_map.is_empty()
    }
}

impl Default for CrossProtocolNetworkAddressTable {
    fn default() -> Self {
        Self {
            addr_map: BiHashMap::new(),
            timeouts: FxHashMap::default(),
        }
    }
}

#[derive(Debug)]
pub struct CrossProtocolNetworkAddressTableWithIpv4Pool {
    /// Internal table
    table: CrossProtocolNetworkAddressTable,
    /// Internal pool of IPv4 prefixes to assign new mappings from
    pool: Vec<Ipv4Net>,
    /// The timeout to use for new entries
    timeout: Duration,
}

impl CrossProtocolNetworkAddressTableWithIpv4Pool {
    /// Construct a new Cross-protocol network address table with a given IPv4 pool
    #[must_use]
    pub fn new(pool: &[Ipv4Net], timeout: Duration) -> Self {
        Self {
            table: CrossProtocolNetworkAddressTable::default(),
            pool: pool.to_vec(),
            timeout,
        }
    }

    /// Insert a new static mapping
    pub fn insert_static(&mut self, ipv4: Ipv4Addr, ipv6: Ipv6Addr) -> Result<(), Error> {
        if !self.pool.iter().any(|prefix| prefix.contains(&ipv4)) {
            return Err(Error::InvalidIpv4Address(ipv4));
        }
        self.table.insert_indefinite(ipv4, ipv6);
        Ok(())
    }

    /// Gets the IPv4 address for a given IPv6 address or inserts a new mapping if one does not exist (if possible)
    pub fn get_or_create_ipv4(&mut self, ipv6: &Ipv6Addr) -> Result<Ipv4Addr, Error> {
        // Return the known mapping if it exists
        if let Some(ipv4) = self.table.get_ipv4(ipv6) {
            return Ok(ipv4);
        }

        // Find the next available IPv4 address in the pool
        let new_address = self
            .pool
            .iter()
            .flat_map(Ipv4Net::hosts)
            .find(|addr| self.table.get_ipv6(addr).is_none())
            .ok_or(Error::Ipv4PoolExhausted)?;

        // Insert the new mapping
        self.table.insert(new_address, *ipv6, self.timeout);
        log::info!(
            "New cross-protocol address mapping: {} -> {}",
            ipv6,
            new_address
        );

        // Return the new address
        Ok(new_address)
    }

    /// Gets the IPv6 address for a given IPv4 address if it exists
    #[must_use]
    pub fn get_ipv6(&self, ipv4: &Ipv4Addr) -> Option<Ipv6Addr> {
        self.table.get_ipv6(ipv4)
    }
}
