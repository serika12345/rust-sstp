#![forbid(unsafe_code)]

mod packet_header;
mod version;

pub use packet_header::{
    EncodedSstpHeader, SstpHeaderDecodeError, SstpPacketHeader, SstpPacketKind, SstpPacketLength,
    SstpPacketLengthError,
};
pub use version::SstpVersion;
