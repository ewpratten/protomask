use lazy_static::lazy_static;
use prometheus::{
    register_int_counter_vec, register_int_gauge, register_int_gauge_vec, IntCounterVec, IntGauge,
    IntGaugeVec,
};

lazy_static! {
    /// Counter for the number of packets processes
    pub static ref PACKET_COUNTER: IntCounterVec = register_int_counter_vec!(
        "packets",
        "Number of packets processed",
        &["protocol", "status"]
    ).unwrap();

    /// Counter for ICMP packet types
    pub static ref ICMP_COUNTER: IntCounterVec = register_int_counter_vec!(
        "icmp",
        "Number of ICMP packets processed",
        &["protocol", "type", "code"]
    ).unwrap();

    /// Gauge for the number of addresses in the IPv4 pool
    pub static ref IPV4_POOL_SIZE: IntGauge = register_int_gauge!(
        "ipv4_pool_size",
        "Number of IPv4 addresses in the pool"
    ).unwrap();

    /// Gauge for the number of addresses currently reserved in the IPv4 pool
    pub static ref IPV4_POOL_RESERVED: IntGaugeVec = register_int_gauge_vec!(
        "ipv4_pool_reserved",
        "Number of IPv4 addresses currently reserved",
        &["static"]
    ).unwrap();
}
