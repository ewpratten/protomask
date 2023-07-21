#[derive(Debug, thiserror::Error)]
pub enum Nat64Error {
    #[error(transparent)]
    Table(#[from] super::table::TableError),
    #[error(transparent)]
    Tun(#[from] protomask_tun::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PacketHandling(#[from] crate::packet::error::PacketError),
    #[error(transparent)]
    PacketReceive(#[from] tokio::sync::broadcast::error::RecvError),
    #[error(transparent)]
    PacketSend(#[from] tokio::sync::mpsc::error::SendError<Vec<u8>>),
}
