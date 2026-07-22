use crate::{SstpHeaderDecodeError, SstpPacketFrame, SstpPacketHeader, SstpPacketLength};
use core::fmt;

/// 逐次復号の一回の呼び出しで消費した入力バイト数です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SstpStreamConsumedBytes {
    value: usize,
}

impl From<usize> for SstpStreamConsumedBytes {
    fn from(value: usize) -> Self {
        Self { value }
    }
}

impl From<SstpStreamConsumedBytes> for usize {
    fn from(value: SstpStreamConsumedBytes) -> Self {
        value.value
    }
}

/// SSTPパケットの逐次復号による一回分の結果です。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SstpPacketStreamStep {
    Incomplete {
        consumed: SstpStreamConsumedBytes,
    },
    Packet {
        packet: SstpPacketFrame,
        consumed: SstpStreamConsumedBytes,
    },
}

impl SstpPacketStreamStep {
    #[must_use]
    pub const fn consumed(&self) -> SstpStreamConsumedBytes {
        match self {
            Self::Incomplete { consumed } | Self::Packet { consumed, .. } => *consumed,
        }
    }
}

/// SSTPパケットの逐次復号が不正ヘッダーを検出した結果です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SstpPacketStreamDecodeError {
    header_error: SstpHeaderDecodeError,
    consumed: SstpStreamConsumedBytes,
}

impl SstpPacketStreamDecodeError {
    const fn new(header_error: SstpHeaderDecodeError, consumed: SstpStreamConsumedBytes) -> Self {
        Self {
            header_error,
            consumed,
        }
    }

    #[must_use]
    pub const fn header_error(self) -> SstpHeaderDecodeError {
        self.header_error
    }

    #[must_use]
    pub const fn consumed(self) -> SstpStreamConsumedBytes {
        self.consumed
    }
}

impl fmt::Display for SstpPacketStreamDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "SSTPパケットを逐次復号できません: {}",
            self.header_error
        )
    }
}

impl std::error::Error for SstpPacketStreamDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.header_error)
    }
}

/// 断片化・結合された入力からSSTPパケットを一個ずつ復号します。
#[derive(Debug, Default)]
pub struct SstpPacketStreamDecoder {
    pending: Vec<u8>,
}

impl SstpPacketStreamDecoder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    pub fn decode_next(
        &mut self,
        input: &[u8],
    ) -> Result<SstpPacketStreamStep, SstpPacketStreamDecodeError> {
        let header_length = usize::from(u16::from(SstpPacketLength::MINIMUM));
        let header_bytes = header_length.saturating_sub(self.pending.len());
        let consumed_for_header = header_bytes.min(input.len());
        self.pending
            .extend_from_slice(&input[..consumed_for_header]);

        let consumed = if self.pending.len() < header_length {
            return Ok(SstpPacketStreamStep::Incomplete {
                consumed: SstpStreamConsumedBytes::from(consumed_for_header),
            });
        } else {
            consumed_for_header
        };

        let header = SstpPacketHeader::try_from(self.pending.as_slice()).map_err(|source| {
            SstpPacketStreamDecodeError::new(source, SstpStreamConsumedBytes::from(consumed))
        })?;
        let packet_length = usize::from(u16::from(header.packet_length()));
        let packet_bytes = packet_length - self.pending.len();
        let remaining_input = &input[consumed_for_header..];
        let consumed_for_packet = packet_bytes.min(remaining_input.len());
        self.pending
            .extend_from_slice(&remaining_input[..consumed_for_packet]);
        let consumed = consumed + consumed_for_packet;

        if self.pending.len() < packet_length {
            return Ok(SstpPacketStreamStep::Incomplete {
                consumed: SstpStreamConsumedBytes::from(consumed),
            });
        }

        let packet =
            SstpPacketFrame::from_validated_parts(header, core::mem::take(&mut self.pending));

        Ok(SstpPacketStreamStep::Packet {
            packet,
            consumed: SstpStreamConsumedBytes::from(consumed),
        })
    }

    #[cfg(test)]
    fn pending_len(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests;
