use std::time::Duration;

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
    pub fn insert_indefinite<T4: Into<u32>, T6: Into<u128>>(&mut self, ipv4: T4, ipv6: T6) {
        self.prune();
        let (ipv4, ipv6) = (ipv4.into(), ipv6.into());
        self.addr_map.insert(ipv4, ipv6);
        self.timeouts.insert((ipv4, ipv6), MaybeTimeout::Never);
    }

    /// Insert a new mapping with a finite time-to-live
    pub fn insert<T4: Into<u32>, T6: Into<u128>>(
        &mut self,
        ipv4: T4,
        ipv6: T6,
        duration: Duration,
    ) {
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
    pub fn get_ipv6<T: Into<u32>>(&self, ipv4: T) -> Option<u128> {
        self.addr_map.get_right(&ipv4.into()).copied()
    }

    /// Get the IPv4 address for a given IPv6 address
    #[must_use]
    pub fn get_ipv4<T: Into<u128>>(&self, ipv6: T) -> Option<u32> {
        self.addr_map.get_left(&ipv6.into()).copied()
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
    pool: Vec<(u32, u32)>,
    /// The timeout to use for new entries
    timeout: Duration,
    /// The pre-calculated maximum number of mappings that can be created
    max_mappings: usize,
}

impl CrossProtocolNetworkAddressTableWithIpv4Pool {
    /// Construct a new Cross-protocol network address table with a given IPv4 pool
    #[must_use]
    pub fn new<T: Into<u32> + Clone>(pool: &[(T, T)], timeout: Duration) -> Self {
        Self {
            table: CrossProtocolNetworkAddressTable::default(),
            pool: pool
                .iter()
                .map(|(a, b)| (a.clone().into(), b.clone().into()))
                .collect(),
            timeout,
            max_mappings: pool
                .iter()
                .map(|(_, netmask)| (*netmask).clone().into() as usize)
                .map(|netmask| !netmask)
                .sum(),
        }
    }

    /// Check if the pool contains an address
    #[must_use]
    pub fn contains<T: Into<u32>>(&self, addr: T) -> bool {
        let addr = addr.into();
        self.pool
            .iter()
            .any(|(network_addr, netmask)| (addr & netmask) == *network_addr)
    }

    /// Insert a new static mapping
    pub fn insert_static<T4: Into<u32>, T6: Into<u128>>(
        &mut self,
        ipv4: T4,
        ipv6: T6,
    ) -> Result<(), Error> {
        let (ipv4, ipv6) = (ipv4.into(), ipv6.into());
        if !self.contains(ipv4) {
            return Err(Error::InvalidIpv4Address(ipv4));
        }
        self.table.insert_indefinite(ipv4, ipv6);
        Ok(())
    }

    /// Gets the IPv4 address for a given IPv6 address or inserts a new mapping if one does not exist (if possible)
    pub fn get_or_create_ipv4<T: Into<u128>>(&mut self, ipv6: T) -> Result<u32, Error> {
        let ipv6 = ipv6.into();

        // Return the known mapping if it exists
        if let Some(ipv4) = self.table.get_ipv4(ipv6) {
            return Ok(ipv4);
        }

        // Otherwise, we first need to make sure there is actually room for a new mapping
        if self.table.len() >= self.max_mappings {
            return Err(Error::Ipv4PoolExhausted(self.max_mappings));
        }

        // Find the next available IPv4 address in the pool
        let new_address = self
            .pool
            .iter()
            .map(|(network_address, netmask)| (*network_address)..(*network_address | !netmask))
            .find_map(|mut addr_range| addr_range.find(|addr| !self.table.get_ipv6(*addr).is_some()))
            .ok_or(Error::Ipv4PoolExhausted(self.max_mappings))?;

        // Insert the new mapping
        self.table.insert(new_address, ipv6, self.timeout);
        log::info!(
            "New cross-protocol address mapping: {:02x} -> {:02x}",
            ipv6,
            new_address
        );

        // Return the new address
        Ok(new_address)
    }

    /// Gets the IPv6 address for a given IPv4 address if it exists
    #[must_use]
    pub fn get_ipv6<T: Into<u32>>(&self, ipv4: T) -> Option<u128> {
        self.table.get_ipv6(ipv4)
    }
}
