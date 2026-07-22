#![forbid(unsafe_code)]

mod data_packet;
mod packet_header;
mod version;

pub use data_packet::{
    EncodedSstpDataPacket, SstpDataPacket, SstpDataPacketDecodeError, SstpDataPayload,
    SstpDataPayloadError,
};
pub use packet_header::{
    EncodedSstpHeader, SstpHeaderDecodeError, SstpPacketHeader, SstpPacketKind, SstpPacketLength,
    SstpPacketLengthError,
};
pub use version::SstpVersion;
