#[derive(Debug, thiserror::Error)]
pub enum Nat64Error {
    #[error(transparent)]
    TableError(#[from] super::table::TableError),
    #[error(transparent)]
    TunError(#[from] protomask_tun::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    PacketHandlingError(#[from] crate::packet::error::PacketError),
    #[error(transparent)]
    PacketReceiveError(#[from] tokio::sync::broadcast::error::RecvError),
    #[error(transparent)]
    PacketSendError(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
}
