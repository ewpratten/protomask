//! Packet type translation functionality

mod icmp;
mod tcp;
mod udp;

pub use icmp::{proxy_icmp_packet, IcmpProxyError};
pub use tcp::{proxy_tcp_packet, TcpProxyError};
// pub use udp::{proxy_udp_packet, UdpProxyError};
pub use udp::{translate_udp_4_to_6, translate_udp_6_to_4};

#[derive(Debug, thiserror::Error)]
pub enum PacketTranslationError {
    #[error("Input packet too short. Got {0} bytes")]
    InputPacketTooShort(usize),
}
