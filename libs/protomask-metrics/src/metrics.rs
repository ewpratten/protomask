use lazy_static::lazy_static;

pub mod label_values {
    /// IPv4 protocol
    pub const PROTOCOL_IPV4: &str = "ipv4";
    /// IPv6 protocol
    pub const PROTOCOL_IPV6: &str = "ipv6";
    /// ICMP protocol
    pub const PROTOCOL_ICMP: &str = "icmp";
    /// ICMPv6 protocol
    pub const PROTOCOL_ICMPV6: &str = "icmpv6";
    /// TCP protocol
    pub const PROTOCOL_TCP: &str = "tcp";
    /// UDP protocol
    pub const PROTOCOL_UDP: &str = "udp";

    /// Dropped status
    pub const STATUS_DROPPED: &str = "dropped";
    /// Translated status
    pub const STATUS_TRANSLATED: &str = "translated";
}

lazy_static! {
    /// Counter for the number of packets processed
    pub static ref PACKET_COUNTER: prometheus::IntCounterVec = prometheus::register_int_counter_vec!(
        "protomask_packets",
        "Number of packets processed",
        &["protocol", "status"]
    ).unwrap();

    /// Counter for the number of different types of ICMP packets received
    pub static ref ICMP_COUNTER: prometheus::IntCounterVec = prometheus::register_int_counter_vec!(
        "protomask_icmp_packets_recv",
        "Number of ICMP packets received",
        &["protocol", "icmp_type", "icmp_code"]
    ).unwrap();
}
