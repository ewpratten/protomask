use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};


#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum PacketStatus {
    Sent,
    Accepted,
    Dropped,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PacketsMetric {
    /// The protocol being counted
    pub protocol: IpProtocol,

    /// The status of the packet
    pub status: PacketStatus,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum IpProtocol {
    Ipv4,
    Ipv6,
}
