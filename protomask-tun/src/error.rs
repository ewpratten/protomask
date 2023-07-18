#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    NetlinkError(#[from] rtnetlink::Error),
}

pub type Result<T> = std::result::Result<T, Error>;