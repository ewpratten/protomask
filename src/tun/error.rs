#[derive(Debug, thiserror::Error)]
pub enum TunError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    NetlinkError(#[from] rtnetlink::Error),
}
