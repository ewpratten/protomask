use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpBimap {
    v4_to_v6: FxHashMap<u32, u128>,
    v6_to_v4: FxHashMap<u128, u32>,
}

impl IpBimap {
    /// Construct a new `IpBimap`
    pub fn new() -> Self {        
        Self {
            v4_to_v6: FxHashMap::default(),
            v6_to_v4: FxHashMap::default(),
        }
    }

    /// Insert a new mapping
    pub fn insert(&mut self, v4: u32, v6: u128) {
        self.v4_to_v6.insert(v4, v6);
        self.v6_to_v4.insert(v6, v4);
    }

    /// Remove a mapping
    pub fn remove(&mut self, v4: u32, v6: u128) {
        self.v4_to_v6.remove(&v4);
        self.v6_to_v4.remove(&v6);
    }

    /// Get the IPv6 address for a given IPv4 address
    pub fn get_v6(&self, v4: u32) -> Option<u128> {
        self.v4_to_v6.get(&v4).copied()
    }

    /// Get the IPv4 address for a given IPv6 address
    pub fn get_v4(&self, v6: u128) -> Option<u32> {
        self.v6_to_v4.get(&v6).copied()
    }

    /// Check if the map contains a given IPv4 address
    pub fn contains_v4(&self, v4: u32) -> bool {
        self.v4_to_v6.contains_key(&v4)
    }

    /// Check if the map contains a given IPv6 address
    pub fn contains_v6(&self, v6: u128) -> bool {
        self.v6_to_v4.contains_key(&v6)
    }
}

impl Default for IpBimap {
    fn default() -> Self {
        Self::new()
    }
}
