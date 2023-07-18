use crate::packet::error::PacketError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawBytes(pub Vec<u8>);

impl TryFrom<Vec<u8>> for RawBytes {
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(bytes))
    }
}

impl Into<Vec<u8>> for RawBytes {
    fn into(self) -> Vec<u8> {
        self.0
    }
}