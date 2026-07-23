#![no_main]

use libfuzzer_sys::fuzz_target;
use sstp_protocol::{
    EncodedSstpDataPacket, EncodedSstpHeader, SstpDataPacket, SstpPacketHeader, SstpPacketKind,
    SstpPacketStreamDecoder, SstpPacketStreamStep,
};

fuzz_target!(|input: &[u8]| {
    check_header_round_trip(input);
    check_data_packet_round_trip(input);
    check_stream_decoder(input);
});

fn check_header_round_trip(input: &[u8]) {
    if let Ok(header) = SstpPacketHeader::try_from(input) {
        let encoded = EncodedSstpHeader::from(header);
        let decoded = SstpPacketHeader::try_from(encoded.as_ref());
        assert_eq!(decoded, Ok(header));
    }
}

fn check_data_packet_round_trip(input: &[u8]) {
    if let Ok(packet) = SstpDataPacket::try_from(input) {
        let expected = packet.clone();
        let encoded = EncodedSstpDataPacket::from(packet);
        let decoded = SstpDataPacket::try_from(encoded.as_ref());
        assert_eq!(decoded, Ok(expected));
    }
}

fn check_stream_decoder(input: &[u8]) {
    check_stream_decoder_with_chunk_size(input, input.len().max(1));
    check_stream_decoder_with_chunk_size(input, 1);
}

fn check_stream_decoder_with_chunk_size(input: &[u8], chunk_size: usize) {
    let mut decoder = SstpPacketStreamDecoder::new();

    if input.is_empty() {
        assert!(matches!(
            decoder.decode_next(input),
            Ok(SstpPacketStreamStep::Incomplete { .. })
        ));
        return;
    }

    let mut input_offset = 0;
    while input_offset < input.len() {
        let chunk_end = input_offset.saturating_add(chunk_size).min(input.len());
        let mut chunk = &input[input_offset..chunk_end];

        while !chunk.is_empty() {
            let step = match decoder.decode_next(chunk) {
                Ok(step) => step,
                Err(error) => {
                    assert!(usize::from(error.consumed()) <= chunk.len());
                    return;
                }
            };
            let consumed = usize::from(step.consumed());
            assert!(consumed <= chunk.len());

            match step {
                SstpPacketStreamStep::Incomplete { .. } => {
                    assert_eq!(consumed, chunk.len());
                }
                SstpPacketStreamStep::Packet { packet, .. } => {
                    assert!(consumed > 0);
                    if packet.kind() == SstpPacketKind::Data {
                        let decoded = SstpDataPacket::try_from(packet);
                        assert!(decoded.is_ok());
                        if let Ok(data_packet) = decoded {
                            let expected = data_packet.clone();
                            let encoded = EncodedSstpDataPacket::from(data_packet);
                            assert_eq!(SstpDataPacket::try_from(encoded.as_ref()), Ok(expected));
                        }
                    }
                }
            }

            input_offset += consumed;
            chunk = &chunk[consumed..];
        }
    }
}
