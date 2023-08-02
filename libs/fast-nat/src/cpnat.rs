use std::time::Duration;

use rustc_hash::FxHashMap;

use crate::{bimap::BiHashMap, timeout::MaybeTimeout};

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
}

impl Default for CrossProtocolNetworkAddressTable {
    fn default() -> Self {
        Self {
            addr_map: BiHashMap::new(),
            timeouts: FxHashMap::default(),
        }
    }
}
