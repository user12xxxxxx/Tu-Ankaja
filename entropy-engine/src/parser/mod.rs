use crate::errors::EntropyError;
use crate::models::EntropyPacket;

/// Packet wire format (MYOSA UART):
///   [0]       magic byte 0xE7
///   [1]       source_id
///   [2..10]   timestamp (little-endian u64)
///   [10..12]  payload_length (little-endian u16)
///   [12..12+N] payload
///   [last 2]  CRC-16 checksum (little-endian u16)
const MAGIC: u8 = 0xE7;
const HEADER_LEN: usize = 12;
const CHECKSUM_LEN: usize = 2;

/// Maximum allowed payload size to prevent memory exhaustion.
const MAX_PAYLOAD_LEN: usize = 4096;

pub struct PacketParser;

impl PacketParser {
    /// Parse a raw UART byte buffer into an `EntropyPacket`.
    ///
    /// The buffer must be exactly the right length for the packet — trailing
    /// bytes are rejected to catch framing errors (M5 fix).
    pub fn parse(raw: &[u8]) -> Result<EntropyPacket, EntropyError> {
        if raw.len() < HEADER_LEN + CHECKSUM_LEN {
            return Err(EntropyError::ParseError {
                reason: format!(
                    "buffer too short: {} bytes, minimum {}",
                    raw.len(),
                    HEADER_LEN + CHECKSUM_LEN
                ),
            });
        }

        if raw[0] != MAGIC {
            return Err(EntropyError::ParseError {
                reason: format!("invalid magic byte: 0x{:02X}, expected 0xE7", raw[0]),
            });
        }

        let source_id = raw[1];
        let timestamp =
            u64::from_le_bytes(
                raw[2..10]
                    .try_into()
                    .map_err(|_| EntropyError::ParseError {
                        reason: "timestamp parse failed".into(),
                    })?,
            );
        let payload_len =
            u16::from_le_bytes(
                raw[10..12]
                    .try_into()
                    .map_err(|_| EntropyError::ParseError {
                        reason: "payload length parse failed".into(),
                    })?,
            ) as usize;

        if payload_len > MAX_PAYLOAD_LEN {
            return Err(EntropyError::ParseError {
                reason: format!(
                    "payload too large: {} bytes, max {}",
                    payload_len, MAX_PAYLOAD_LEN
                ),
            });
        }

        let expected_total = HEADER_LEN + payload_len + CHECKSUM_LEN;
        if raw.len() < expected_total {
            return Err(EntropyError::ParseError {
                reason: format!(
                    "buffer too short for payload: have {}, need {}",
                    raw.len(),
                    expected_total
                ),
            });
        }

        // M5 fix: reject trailing bytes to catch UART framing errors.
        if raw.len() > expected_total {
            return Err(EntropyError::ParseError {
                reason: format!(
                    "unexpected trailing bytes: have {}, expected {}",
                    raw.len(),
                    expected_total
                ),
            });
        }

        let payload = raw[HEADER_LEN..HEADER_LEN + payload_len].to_vec();
        let checksum = u16::from_le_bytes(
            raw[HEADER_LEN + payload_len..HEADER_LEN + payload_len + 2]
                .try_into()
                .map_err(|_| EntropyError::ParseError {
                    reason: "checksum parse failed".into(),
                })?,
        );

        let packet = EntropyPacket {
            source_id,
            timestamp,
            payload,
            checksum,
        };

        if !Self::validate_checksum(&packet, &raw[..HEADER_LEN + payload_len]) {
            return Err(EntropyError::ParseError {
                reason: "CRC-16 checksum mismatch".into(),
            });
        }

        Ok(packet)
    }

    /// Validate the CRC-16 checksum against the header+payload bytes.
    pub fn validate_checksum(packet: &EntropyPacket, data: &[u8]) -> bool {
        Self::compute_crc16(data) == packet.checksum
    }

    /// CRC-16/CCITT-FALSE used by MYOSA firmware.
    fn compute_crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= u16::from(byte) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// Build a raw packet buffer (for testing / simulation).
    ///
    /// Panics if payload exceeds `MAX_PAYLOAD_LEN` (M6 fix).
    pub fn build_raw(source_id: u8, timestamp: u64, payload: &[u8]) -> Vec<u8> {
        assert!(
            payload.len() <= MAX_PAYLOAD_LEN,
            "payload size {} exceeds max {}",
            payload.len(),
            MAX_PAYLOAD_LEN
        );

        let mut buf = Vec::with_capacity(HEADER_LEN + payload.len() + CHECKSUM_LEN);
        buf.push(MAGIC);
        buf.push(source_id);
        buf.extend_from_slice(&timestamp.to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        buf.extend_from_slice(payload);
        let crc = Self::compute_crc16(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_parse() {
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x42];
        let raw = PacketParser::build_raw(0x01, 1000, &payload);
        let packet = PacketParser::parse(&raw).unwrap();
        assert_eq!(packet.source_id, 0x01);
        assert_eq!(packet.timestamp, 1000);
        assert_eq!(packet.payload, payload);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut raw = PacketParser::build_raw(0x01, 0, &[0xFF]);
        raw[0] = 0x00;
        assert!(PacketParser::parse(&raw).is_err());
    }

    #[test]
    fn rejects_truncated() {
        assert!(PacketParser::parse(&[0xE7, 0x01]).is_err());
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut raw = PacketParser::build_raw(0x01, 0, &[0xAA]);
        raw.push(0xFF); // extra trailing byte
        let err = PacketParser::parse(&raw);
        assert!(err.is_err(), "must reject packets with trailing bytes");
    }

    #[test]
    #[should_panic(expected = "payload size")]
    fn build_raw_panics_on_oversized_payload() {
        let big = vec![0u8; MAX_PAYLOAD_LEN + 1];
        PacketParser::build_raw(0x01, 0, &big);
    }

    #[test]
    fn rejects_corrupted_checksum() {
        let mut raw = PacketParser::build_raw(0x01, 0, &[0xAA, 0xBB, 0xCC]);
        // Corrupt the last byte (checksum).
        let last = raw.len() - 1;
        raw[last] ^= 0xFF;
        let err = PacketParser::parse(&raw);
        assert!(err.is_err(), "must reject corrupted checksum");
    }

    #[test]
    fn empty_payload_round_trip() {
        let raw = PacketParser::build_raw(0x01, 42, &[]);
        let packet = PacketParser::parse(&raw).unwrap();
        assert!(packet.payload.is_empty());
        assert_eq!(packet.timestamp, 42);
    }

    #[test]
    fn max_payload_round_trip() {
        let payload = vec![0xAB; MAX_PAYLOAD_LEN];
        let raw = PacketParser::build_raw(0xFF, u64::MAX, &payload);
        let packet = PacketParser::parse(&raw).unwrap();
        assert_eq!(packet.payload.len(), MAX_PAYLOAD_LEN);
        assert_eq!(packet.source_id, 0xFF);
        assert_eq!(packet.timestamp, u64::MAX);
    }

    #[test]
    fn rejects_claimed_payload_larger_than_max() {
        // Craft a packet that claims a payload larger than MAX_PAYLOAD_LEN.
        let mut raw = vec![0xE7, 0x01]; // magic, source_id
        raw.extend_from_slice(&0u64.to_le_bytes()); // timestamp
        raw.extend_from_slice(&((MAX_PAYLOAD_LEN as u16 + 1).to_le_bytes())); // payload_len > max
        raw.extend(vec![0u8; MAX_PAYLOAD_LEN + 1]); // payload
        raw.extend_from_slice(&0u16.to_le_bytes()); // fake checksum
        assert!(PacketParser::parse(&raw).is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(PacketParser::parse(&[]).is_err());
    }
}
