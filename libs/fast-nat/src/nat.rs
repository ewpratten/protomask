use crate::{bimap::BiHashMap, timeout::MaybeTimeout};
use rustc_hash::FxHashMap;
use std::{net::Ipv4Addr, time::Duration};

/// A table of network address mappings
#[derive(Debug)]
pub struct NetworkAddressTable {
    /// Internal address map
    addr_map: BiHashMap<u32, u32>,
    /// Secondary map used to keep track of timeouts
    timeouts: FxHashMap<(u32, u32), MaybeTimeout>,
}

impl NetworkAddressTable {
    /// Construct a new empty `NetworkAddressTable`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Prune all old mappings
    #[profiling::function]
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
    #[profiling::function]
    pub fn insert_indefinite(&mut self, left: Ipv4Addr, right: Ipv4Addr) {
        self.prune();
        let (left, right) = (left.into(), right.into());
        self.addr_map.insert(left, right);
        self.timeouts.insert((left, right), MaybeTimeout::Never);
    }

    /// Insert a new mapping with a finite time-to-live
    #[profiling::function]
    pub fn insert(&mut self, left: Ipv4Addr, right: Ipv4Addr, duration: Duration) {
        self.prune();
        let (left, right) = (left.into(), right.into());
        self.addr_map.insert(left, right);
        self.timeouts.insert(
            (left, right),
            MaybeTimeout::After {
                duration,
                start: std::time::Instant::now(),
            },
        );
    }

    /// Get the right value for a given left value
    #[must_use]
    #[profiling::function]
    pub fn get_right(&self, left: &Ipv4Addr) -> Option<Ipv4Addr> {
        self.addr_map
            .get_right(&(*left).into())
            .map(|addr| (*addr).into())
    }

    /// Get the left value for a given right value
    #[must_use]
    #[profiling::function]
    pub fn get_left(&self, right: &Ipv4Addr) -> Option<Ipv4Addr> {
        self.addr_map
            .get_left(&(*right).into())
            .map(|addr| (*addr).into())
    }
}

impl Default for NetworkAddressTable {
    fn default() -> Self {
        Self {
            addr_map: BiHashMap::new(),
            timeouts: FxHashMap::default(),
        }
    }
}
