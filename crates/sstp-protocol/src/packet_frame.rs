use crate::{SstpPacketHeader, SstpPacketKind};

/// ヘッダーのLengthとバイト数が一致する一個のSSTPパケットです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SstpPacketFrame {
    header: SstpPacketHeader,
    bytes: Vec<u8>,
}

impl SstpPacketFrame {
    pub(crate) const fn from_validated_parts(header: SstpPacketHeader, bytes: Vec<u8>) -> Self {
        Self { header, bytes }
    }

    #[must_use]
    pub const fn kind(&self) -> SstpPacketKind {
        self.header.kind()
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
