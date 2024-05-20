use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::{Duration, SystemTime},
};

use bimap::BiHashMap;
use ipnet::Ipv4Net;

#[derive(Debug, thiserror::Error)]
pub enum NatTableError {
    #[error("IPv4 address {0} is already in the table")]
    Ipv4AlreadyInTable(Ipv4Addr),
    #[error("IPv6 address {0} is already in the table")]
    Ipv6AlreadyInTable(Ipv6Addr),
}

struct IpMappingLease {
    /// Total number of times this lease has been "ticked"
    ticks: u128,

    /// Time that this lease will expire
    expiry: Option<SystemTime>,
}

pub struct NetworkAddressTranslationTable {
    /// Mapping between addresses
    addr_pairs: BiHashMap<Ipv4Addr, Ipv6Addr>,

    /// Lease duration
    lease_duration: Duration,

    /// Information about leases
    leases: HashMap<(Ipv4Addr, Ipv6Addr), IpMappingLease>,
}

impl NetworkAddressTranslationTable {
    /// Construct a new Cross-protocol network address table
    pub fn new(lease_duration: Duration) -> Self {
        Self {
            addr_pairs: BiHashMap::new(),
            lease_duration,
            leases: HashMap::new(),
        }
    }

    /// Add a pair of addresses to the pool
    pub fn add_pair(
        &mut self,
        ipv4: Ipv4Addr,
        ipv6: Ipv6Addr,
        expires: bool,
    ) -> Result<(), NatTableError> {
        // If either address is already in the table, throw an error
        if self.addr_pairs.contains_left(&ipv4) {
            return Err(NatTableError::Ipv4AlreadyInTable(ipv4));
        }
        if self.addr_pairs.contains_right(&ipv6) {
            return Err(NatTableError::Ipv6AlreadyInTable(ipv6));
        }

        // Insert the pair into the table
        self.addr_pairs.insert(ipv4, ipv6);

        // Add a lease to the lease table
        self.leases.insert(
            (ipv4, ipv6),
            IpMappingLease {
                ticks: 0,
                expiry: if expires {
                    Some(SystemTime::now() + self.lease_duration)
                } else {
                    None
                },
            },
        );

        Ok(())
    }

    /// Get the corresponding IPv6 address for a given IPv4 address
    pub fn get_ipv6(&self, ipv4: &Ipv4Addr) -> Option<Ipv6Addr> {
        self.addr_pairs.get_by_left(ipv4).map(|addr| *addr)
    }

    /// Get the corresponding IPv4 address for a given IPv6 address
    pub fn get_ipv4(&self, ipv6: &Ipv6Addr) -> Option<Ipv4Addr> {
        self.addr_pairs.get_by_right(ipv6).map(|addr| *addr)
    }

    /// "tick" a flow. Allowing it to continue living
    pub fn tick_flow(&mut self, ip: &IpAddr) {
        // Figure out the whole address pair
        let (ipv4, ipv6) = match ip {
            IpAddr::V4(ipv4) => {
                let ipv6 = self.get_ipv6(ipv4).unwrap();
                (ipv4.clone(), ipv6)
            }
            IpAddr::V6(ipv6) => {
                let ipv4 = self.get_ipv4(ipv6).unwrap();
                (ipv4, ipv6.clone())
            }
        };

        // Get the lease
        let lease = self.leases.get_mut(&(ipv4, ipv6)).unwrap();

        // Increment the tick count
        lease.ticks += 1;

        // If the lease has an expiry, update it
        if let Some(_) = lease.expiry {
            lease.expiry = Some(SystemTime::now() + self.lease_duration);
        }
    }

    /// Prune any expired leases
    pub fn prune(&mut self) {
        // Get the current time
        let now = SystemTime::now();

        // Filter out any leases that have expired
        self.leases.retain(|_, lease| {
            // If the lease has no expiry, keep it
            if lease.expiry.is_none() {
                return true;
            }

            // If the lease has expired, remove it
            lease.expiry.unwrap() > now
        });

        // Remove any address pairs that no longer have a lease
        self.addr_pairs
            .retain(|ipv4, ipv6| self.leases.contains_key(&(*ipv4, *ipv6)));
    }

    /// Finds a free IPv4 address within a set of prefixes
    pub fn find_free_ipv4(&self, prefixes: &Vec<Ipv4Net>) -> Option<Ipv4Addr> {
        // Iterate over the prefixes
        for prefix in prefixes {
            // Iterate over all addresses in the prefix
            for addr in prefix.hosts() {
                // If the address is not in the table, return it
                if !self.addr_pairs.contains_left(&addr) {
                    return Some(addr);
                }
            }
        }

        None
    }
}
