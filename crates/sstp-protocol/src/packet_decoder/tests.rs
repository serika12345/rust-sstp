use super::{
    SstpPacketStreamDecodeError, SstpPacketStreamDecoder, SstpPacketStreamStep,
    SstpStreamConsumedBytes,
};
use crate::{SstpDataPacket, SstpHeaderDecodeError, SstpPacketKind};

const FIRST_PACKET: [u8; 8] = [0x10, 0x00, 0x00, 0x08, 0xff, 0x03, 0xc0, 0x21];
const SECOND_PACKET: [u8; 4] = [0x10, 0x00, 0x00, 0x04];

#[test]
fn empty_input_is_incomplete_without_consumption() -> Result<(), SstpPacketStreamDecodeError> {
    let mut decoder = SstpPacketStreamDecoder::new();

    assert_eq!(
        decoder.decode_next(&[])?,
        SstpPacketStreamStep::Incomplete {
            consumed: SstpStreamConsumedBytes::from(0),
        }
    );
    Ok(())
}

#[test]
fn incomplete_input_is_not_an_error() -> Result<(), SstpPacketStreamDecodeError> {
    let mut decoder = SstpPacketStreamDecoder::new();

    assert_eq!(
        decoder.decode_next(&FIRST_PACKET[..2])?,
        SstpPacketStreamStep::Incomplete {
            consumed: SstpStreamConsumedBytes::from(2),
        }
    );
    assert_eq!(decoder.pending_len(), 2);
    Ok(())
}

#[test]
fn control_packet_boundary_is_preserved_without_interpreting_message()
-> Result<(), Box<dyn std::error::Error>> {
    let mut decoder = SstpPacketStreamDecoder::new();
    let step = decoder.decode_next(&[0x10, 0x01, 0x00, 0x08, 0x00, 0x08, 0x00, 0x00])?;

    match step {
        SstpPacketStreamStep::Packet { packet, .. } => {
            assert_eq!(packet.kind(), SstpPacketKind::Control);
            Ok(())
        }
        SstpPacketStreamStep::Incomplete { .. } => Err("control packet was incomplete".into()),
    }
}

#[test]
fn reports_consumed_input_for_combined_packets() -> Result<(), Box<dyn std::error::Error>> {
    let input = [FIRST_PACKET.as_slice(), SECOND_PACKET.as_slice()].concat();
    let mut decoder = SstpPacketStreamDecoder::new();

    let first = decoder.decode_next(&input)?;
    assert_eq!(first.consumed(), SstpStreamConsumedBytes::from(8));
    assert_eq!(packet_payload(first)?, &[0xff, 0x03, 0xc0, 0x21]);

    let second = decoder.decode_next(&input[8..])?;
    assert_eq!(second.consumed(), SstpStreamConsumedBytes::from(4));
    assert!(packet_payload(second)?.is_empty());
    Ok(())
}

#[test]
fn every_partition_produces_the_same_packets() -> Result<(), Box<dyn std::error::Error>> {
    let input = [FIRST_PACKET.as_slice(), SECOND_PACKET.as_slice()].concat();
    let expected = vec![vec![0xff, 0x03, 0xc0, 0x21], Vec::new()];

    for partition_mask in 0..(1_u16 << (input.len() - 1)) {
        assert_eq!(
            decode_partition(&input, partition_mask)?,
            expected,
            "partition mask {partition_mask:#013b}"
        );
    }
    Ok(())
}

#[test]
fn maximum_packet_never_exceeds_accumulation_limit() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = vec![0x10, 0x00, 0x0f, 0xff];
    input.resize(4095, 0xa5);
    let mut decoder = SstpPacketStreamDecoder::new();
    let mut packet = None;

    for byte in input {
        let step = decoder.decode_next(&[byte])?;
        assert!(decoder.pending_len() <= 4095);
        if let SstpPacketStreamStep::Packet {
            packet: decoded, ..
        } = step
        {
            packet = Some(decoded);
        }
    }

    assert_eq!(
        SstpDataPacket::try_from(packet.ok_or("packet was not decoded")?)?
            .payload()
            .as_ref()
            .len(),
        4091
    );
    assert_eq!(decoder.pending_len(), 0);
    Ok(())
}

#[test]
fn malformed_input_is_distinct_from_incomplete_input() -> Result<(), &'static str> {
    let mut decoder = SstpPacketStreamDecoder::new();

    let error = match decoder.decode_next(&[0x11, 0x00, 0x00, 0x04]) {
        Err(error) => error,
        Ok(_) => return Err("unsupported version was accepted"),
    };

    assert_eq!(
        error.header_error(),
        SstpHeaderDecodeError::UnsupportedVersion
    );
    assert_eq!(error.consumed(), SstpStreamConsumedBytes::from(4));
    Ok(())
}

fn decode_partition(
    input: &[u8],
    partition_mask: u16,
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut decoder = SstpPacketStreamDecoder::new();
    let mut payloads = Vec::new();
    let mut chunk_start = 0;

    for boundary in 1..=input.len() {
        let is_partition =
            boundary == input.len() || partition_mask & (1_u16 << (boundary - 1)) != 0;
        if is_partition {
            decode_chunk(&mut decoder, &input[chunk_start..boundary], &mut payloads)?;
            chunk_start = boundary;
        }
    }

    Ok(payloads)
}

fn decode_chunk(
    decoder: &mut SstpPacketStreamDecoder,
    mut chunk: &[u8],
    payloads: &mut Vec<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    while !chunk.is_empty() {
        let step = decoder.decode_next(chunk)?;
        let consumed = usize::from(step.consumed());
        if let SstpPacketStreamStep::Packet { packet, .. } = step {
            payloads.push(
                SstpDataPacket::try_from(packet)?
                    .payload()
                    .as_ref()
                    .to_vec(),
            );
        }
        chunk = &chunk[consumed..];
    }
    Ok(())
}

fn packet_payload(step: SstpPacketStreamStep) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match step {
        SstpPacketStreamStep::Packet { packet, .. } => Ok(SstpDataPacket::try_from(packet)?
            .payload()
            .as_ref()
            .to_vec()),
        SstpPacketStreamStep::Incomplete { .. } => Err("packet is incomplete".into()),
    }
}
