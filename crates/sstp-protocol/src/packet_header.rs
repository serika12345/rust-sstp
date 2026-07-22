use crate::SstpVersion;
use core::fmt;

const SSTP_VERSION_1_0: u8 = 0x10;
const HEADER_LENGTH: usize = 4;
const CONTROL_PACKET_BIT: u8 = 0x01;
const PACKET_LENGTH_MASK: u16 = 0x0fff;

/// SSTPパケットが制御情報とデータのどちらを運ぶかを表します。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpPacketKind {
    Data,
    Control,
}

/// SSTPヘッダーを含むパケット全体のバイト長です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SstpPacketLength {
    value: u16,
}

impl SstpPacketLength {
    pub const MINIMUM: Self = Self { value: 4 };
    pub const MAXIMUM: Self = Self {
        value: PACKET_LENGTH_MASK,
    };
}

/// SSTPパケット長を構築できなかった理由です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpPacketLengthError {
    TooShort,
    TooLong,
}

impl fmt::Display for SstpPacketLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => formatter.write_str("SSTPパケット長がヘッダー長を下回っています"),
            Self::TooLong => formatter.write_str("SSTPパケット長が12ビットの上限を超えています"),
        }
    }
}

impl std::error::Error for SstpPacketLengthError {}

impl TryFrom<u16> for SstpPacketLength {
    type Error = SstpPacketLengthError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < Self::MINIMUM.value {
            return Err(SstpPacketLengthError::TooShort);
        }
        if value > Self::MAXIMUM.value {
            return Err(SstpPacketLengthError::TooLong);
        }

        Ok(Self { value })
    }
}

impl From<SstpPacketLength> for u16 {
    fn from(value: SstpPacketLength) -> Self {
        value.value
    }
}

/// 検証済みのSSTP通信ヘッダーです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SstpPacketHeader {
    version: SstpVersion,
    kind: SstpPacketKind,
    packet_length: SstpPacketLength,
}

impl SstpPacketHeader {
    #[must_use]
    pub const fn new(kind: SstpPacketKind, packet_length: SstpPacketLength) -> Self {
        Self {
            version: SstpVersion::V1_0,
            kind,
            packet_length,
        }
    }

    #[must_use]
    pub const fn version(self) -> SstpVersion {
        self.version
    }

    #[must_use]
    pub const fn kind(self) -> SstpPacketKind {
        self.kind
    }

    #[must_use]
    pub const fn packet_length(self) -> SstpPacketLength {
        self.packet_length
    }
}

/// SSTP通信ヘッダーを復号できなかった理由です。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpHeaderDecodeError {
    IncompleteHeader,
    UnsupportedVersion,
    PacketLengthTooShort,
}

impl fmt::Display for SstpHeaderDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteHeader => formatter.write_str("SSTPヘッダーが4バイト未満です"),
            Self::UnsupportedVersion => formatter.write_str("SSTP versionはサポート対象外です"),
            Self::PacketLengthTooShort => {
                formatter.write_str("SSTPパケット長がヘッダー長を下回っています")
            }
        }
    }
}

impl std::error::Error for SstpHeaderDecodeError {}

impl TryFrom<&[u8]> for SstpPacketHeader {
    type Error = SstpHeaderDecodeError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < HEADER_LENGTH {
            return Err(SstpHeaderDecodeError::IncompleteHeader);
        }
        if bytes[0] != SSTP_VERSION_1_0 {
            return Err(SstpHeaderDecodeError::UnsupportedVersion);
        }

        let kind = if bytes[1] & CONTROL_PACKET_BIT == 0 {
            SstpPacketKind::Data
        } else {
            SstpPacketKind::Control
        };
        let encoded_length = u16::from_be_bytes([bytes[2], bytes[3]]);
        let packet_length = SstpPacketLength::try_from(encoded_length & PACKET_LENGTH_MASK)
            .map_err(|_| SstpHeaderDecodeError::PacketLengthTooShort)?;

        Ok(Self::new(kind, packet_length))
    }
}

/// ネットワークへ送信できる4バイトのSSTP通信ヘッダーです。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodedSstpHeader {
    bytes: [u8; HEADER_LENGTH],
}

impl From<SstpPacketHeader> for EncodedSstpHeader {
    fn from(header: SstpPacketHeader) -> Self {
        let version = match header.version {
            SstpVersion::V1_0 => SSTP_VERSION_1_0,
        };
        let kind = match header.kind {
            SstpPacketKind::Data => 0,
            SstpPacketKind::Control => CONTROL_PACKET_BIT,
        };
        let length_bytes = u16::from(header.packet_length).to_be_bytes();

        Self {
            bytes: [version, kind, length_bytes[0], length_bytes[1]],
        }
    }
}

impl AsRef<[u8]> for EncodedSstpHeader {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EncodedSstpHeader, SstpHeaderDecodeError, SstpPacketHeader, SstpPacketKind,
        SstpPacketLength, SstpPacketLengthError,
    };
    use crate::SstpVersion;

    #[test]
    fn decodes_minimum_data_header() -> Result<(), SstpHeaderDecodeError> {
        let header = SstpPacketHeader::try_from(&[0x10, 0x00, 0x00, 0x04][..])?;

        assert_eq!(header.version(), SstpVersion::V1_0);
        assert_eq!(header.kind(), SstpPacketKind::Data);
        assert_eq!(header.packet_length(), SstpPacketLength::MINIMUM);
        Ok(())
    }

    #[test]
    fn encodes_control_header_in_network_byte_order() -> Result<(), SstpPacketLengthError> {
        let length = SstpPacketLength::try_from(0x0123)?;
        let header = SstpPacketHeader::new(SstpPacketKind::Control, length);
        let encoded = EncodedSstpHeader::from(header);

        assert_eq!(encoded.as_ref(), &[0x10, 0x01, 0x01, 0x23]);
        Ok(())
    }

    #[test]
    fn maximum_length_header_round_trips() -> Result<(), Box<dyn std::error::Error>> {
        let length = SstpPacketLength::try_from(0x0fff)?;
        let header = SstpPacketHeader::new(SstpPacketKind::Data, length);
        let encoded = EncodedSstpHeader::from(header);

        assert_eq!(encoded.as_ref(), &[0x10, 0x00, 0x0f, 0xff]);
        assert_eq!(SstpPacketHeader::try_from(encoded.as_ref())?, header);
        Ok(())
    }

    #[test]
    fn rejects_incomplete_header() {
        for bytes in [
            &[][..],
            &[0x10][..],
            &[0x10, 0x00][..],
            &[0x10, 0x00, 0x00][..],
        ] {
            assert_eq!(
                SstpPacketHeader::try_from(bytes),
                Err(SstpHeaderDecodeError::IncompleteHeader)
            );
        }
    }

    #[test]
    fn rejects_unsupported_version() {
        assert_eq!(
            SstpPacketHeader::try_from(&[0x11, 0x00, 0x00, 0x04][..]),
            Err(SstpHeaderDecodeError::UnsupportedVersion)
        );
    }

    #[test]
    fn rejects_declared_length_shorter_than_header() {
        assert_eq!(
            SstpPacketHeader::try_from(&[0x10, 0x00, 0x00, 0x03][..]),
            Err(SstpHeaderDecodeError::PacketLengthTooShort)
        );
    }

    #[test]
    fn ignores_reserved_bits_on_receipt() -> Result<(), SstpHeaderDecodeError> {
        let header = SstpPacketHeader::try_from(&[0x10, 0xfe, 0xf0, 0x04][..])?;

        assert_eq!(header.kind(), SstpPacketKind::Data);
        assert_eq!(header.packet_length(), SstpPacketLength::MINIMUM);
        Ok(())
    }

    #[test]
    fn encoding_clears_reserved_bits() -> Result<(), SstpHeaderDecodeError> {
        let received = SstpPacketHeader::try_from(&[0x10, 0xff, 0xf0, 0x08][..])?;
        let encoded = EncodedSstpHeader::from(received);

        assert_eq!(encoded.as_ref(), &[0x10, 0x01, 0x00, 0x08]);
        Ok(())
    }

    #[test]
    fn packet_length_rejects_value_shorter_than_header() {
        assert_eq!(
            SstpPacketLength::try_from(3),
            Err(SstpPacketLengthError::TooShort)
        );
    }

    #[test]
    fn packet_length_rejects_value_larger_than_twelve_bits() {
        assert_eq!(
            SstpPacketLength::try_from(0x1000),
            Err(SstpPacketLengthError::TooLong)
        );
    }
}
