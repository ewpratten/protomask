// Calling send with nested enums is kinda messy, so these macros clean up the calling code a bit

#[macro_export]
#[cfg_attr(rustfmt, rustfmt_skip)]
macro_rules! count_packet {
    ($sender: expr, $protocol: expr, $status: expr) => {
        $sender.send(
            crate::metrics::registry::MetricEvent::CounterAdd(
                crate::metrics::registry::Metric::Packets(
                    crate::metrics::labels::PacketsMetric {
                        protocol: $protocol,
                        status: $status,
                    }
                ),
            )
        ).await
    };
}

#[macro_export]
macro_rules! count_packet_ipv4 {
    ($sender: expr, $status: expr) => {
        count_packet!($sender, crate::metrics::labels::IpProtocol::Ipv4, $status)
    };
}

#[macro_export]
macro_rules! count_packet_ipv6 {
    ($sender: expr, $status: expr) => {
        count_packet!($sender, crate::metrics::labels::IpProtocol::Ipv6, $status)
    };
}