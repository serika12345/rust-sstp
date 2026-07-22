use crate::{
    EncodedSstpHeader, SstpHeaderDecodeError, SstpPacketFrame, SstpPacketHeader, SstpPacketKind,
    SstpPacketLength,
};
use core::cmp::Ordering;
use core::fmt;

/// SSTP Data Packetが所有する上位プロトコルのpayloadです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SstpDataPayload {
    bytes: Vec<u8>,
    packet_length: SstpPacketLength,
}

/// SSTP Data Packetのpayloadを構築できなかった理由です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpDataPayloadError {
    TooLong,
}

impl fmt::Display for SstpDataPayloadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong => formatter.write_str("SSTP Data Packetのpayloadが長すぎます"),
        }
    }
}

impl std::error::Error for SstpDataPayloadError {}

impl TryFrom<Vec<u8>> for SstpDataPayload {
    type Error = SstpDataPayloadError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let header_length = usize::from(u16::from(SstpPacketLength::MINIMUM));
        let maximum_packet_length = usize::from(u16::from(SstpPacketLength::MAXIMUM));
        if bytes.len() > maximum_packet_length - header_length {
            return Err(SstpDataPayloadError::TooLong);
        }

        let total_length = u16::try_from(header_length + bytes.len())
            .map_err(|_| SstpDataPayloadError::TooLong)?;
        let packet_length =
            SstpPacketLength::try_from(total_length).map_err(|_| SstpDataPayloadError::TooLong)?;

        Ok(Self {
            bytes,
            packet_length,
        })
    }
}

impl AsRef<[u8]> for SstpDataPayload {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// 一個の完全なSSTP Data Packetです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SstpDataPacket {
    header: SstpPacketHeader,
    payload: SstpDataPayload,
}

impl SstpDataPacket {
    #[must_use]
    pub const fn new(payload: SstpDataPayload) -> Self {
        Self {
            header: SstpPacketHeader::new(SstpPacketKind::Data, payload.packet_length),
            payload,
        }
    }

    #[must_use]
    pub const fn payload(&self) -> &SstpDataPayload {
        &self.payload
    }
}

/// 一個の完全なSSTP Data Packetを復号できなかった理由です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpDataPacketDecodeError {
    Header(SstpHeaderDecodeError),
    ExpectedDataPacket,
    Truncated,
    TrailingBytes,
}

impl fmt::Display for SstpDataPacketDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header(source) => write!(formatter, "SSTPヘッダーを復号できません: {source}"),
            Self::ExpectedDataPacket => formatter.write_str("SSTP Data Packetではありません"),
            Self::Truncated => formatter.write_str("SSTP Data Packetが宣言長より短いです"),
            Self::TrailingBytes => {
                formatter.write_str("SSTP Data Packetの後ろに余剰バイトがあります")
            }
        }
    }
}

impl std::error::Error for SstpDataPacketDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Header(source) => Some(source),
            Self::ExpectedDataPacket | Self::Truncated | Self::TrailingBytes => None,
        }
    }
}

impl From<SstpHeaderDecodeError> for SstpDataPacketDecodeError {
    fn from(source: SstpHeaderDecodeError) -> Self {
        Self::Header(source)
    }
}

impl TryFrom<&[u8]> for SstpDataPacket {
    type Error = SstpDataPacketDecodeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let header = SstpPacketHeader::try_from(bytes)?;
        if header.kind() != SstpPacketKind::Data {
            return Err(SstpDataPacketDecodeError::ExpectedDataPacket);
        }

        let declared_length = usize::from(u16::from(header.packet_length()));
        match bytes.len().cmp(&declared_length) {
            Ordering::Less => return Err(SstpDataPacketDecodeError::Truncated),
            Ordering::Greater => return Err(SstpDataPacketDecodeError::TrailingBytes),
            Ordering::Equal => {}
        }

        let header_length = usize::from(u16::from(SstpPacketLength::MINIMUM));
        let payload = SstpDataPayload {
            bytes: bytes[header_length..].to_vec(),
            packet_length: header.packet_length(),
        };

        Ok(Self { header, payload })
    }
}

impl TryFrom<SstpPacketFrame> for SstpDataPacket {
    type Error = SstpDataPacketDecodeError;

    fn try_from(frame: SstpPacketFrame) -> Result<Self, Self::Error> {
        Self::try_from(frame.as_bytes())
    }
}

/// ネットワークへ送信できる一個の完全なSSTP Data Packetです。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedSstpDataPacket {
    bytes: Vec<u8>,
}

impl From<SstpDataPacket> for EncodedSstpDataPacket {
    fn from(packet: SstpDataPacket) -> Self {
        let mut bytes = Vec::with_capacity(usize::from(u16::from(packet.header.packet_length())));
        bytes.extend_from_slice(EncodedSstpHeader::from(packet.header).as_ref());
        bytes.extend_from_slice(packet.payload.as_ref());
        Self { bytes }
    }
}

impl AsRef<[u8]> for EncodedSstpDataPacket {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EncodedSstpDataPacket, SstpDataPacket, SstpDataPacketDecodeError, SstpDataPayload,
        SstpDataPayloadError,
    };

    #[test]
    fn decodes_complete_data_packet() -> Result<(), SstpDataPacketDecodeError> {
        let packet =
            SstpDataPacket::try_from(&[0x10, 0x00, 0x00, 0x08, 0xff, 0x03, 0xc0, 0x21][..])?;

        assert_eq!(packet.payload().as_ref(), &[0xff, 0x03, 0xc0, 0x21]);
        Ok(())
    }

    #[test]
    fn minimum_empty_data_packet_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        let payload = SstpDataPayload::try_from(Vec::new())?;
        let packet = SstpDataPacket::new(payload);
        let encoded = EncodedSstpDataPacket::from(packet);

        assert_eq!(encoded.as_ref(), &[0x10, 0x00, 0x00, 0x04]);
        assert!(
            SstpDataPacket::try_from(encoded.as_ref())?
                .payload()
                .as_ref()
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn maximum_data_packet_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        let payload = SstpDataPayload::try_from(vec![0xa5; 4091])?;
        let packet = SstpDataPacket::new(payload);
        let encoded = EncodedSstpDataPacket::from(packet);

        assert_eq!(encoded.as_ref().len(), 4095);
        assert_eq!(&encoded.as_ref()[..4], &[0x10, 0x00, 0x0f, 0xff]);
        assert_eq!(
            SstpDataPacket::try_from(encoded.as_ref())?
                .payload()
                .as_ref(),
            vec![0xa5; 4091]
        );
        Ok(())
    }

    #[test]
    fn rejects_truncated_data_packet() {
        assert_eq!(
            SstpDataPacket::try_from(&[0x10, 0x00, 0x00, 0x08, 0xff, 0x03, 0xc0][..]),
            Err(SstpDataPacketDecodeError::Truncated)
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        assert_eq!(
            SstpDataPacket::try_from(&[0x10, 0x00, 0x00, 0x04, 0x00][..]),
            Err(SstpDataPacketDecodeError::TrailingBytes)
        );
    }

    #[test]
    fn rejects_control_packet() {
        assert_eq!(
            SstpDataPacket::try_from(&[0x10, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00][..]),
            Err(SstpDataPacketDecodeError::ExpectedDataPacket)
        );
    }

    #[test]
    fn payload_rejects_value_larger_than_packet_capacity() {
        assert_eq!(
            SstpDataPayload::try_from(vec![0; 4092]),
            Err(SstpDataPayloadError::TooLong)
        );
    }
}
