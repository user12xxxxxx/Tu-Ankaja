use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::errors::EntropyError;
use crate::models::EntropyPacket;
use crate::parser::PacketParser;

/// Maximum bytes that can be requested in a single read.
const MAX_READ_LEN: usize = 1 << 20; // 1 MiB

/// Default baud rate for MYOSA UART communication.
const DEFAULT_BAUD_RATE: u32 = 115_200;

/// Read timeout for serial port operations.
const SERIAL_TIMEOUT: Duration = Duration::from_secs(2);

enum SourceMode {
    Simulated { state: u64 },
    Hardware { port: Box<dyn serialport::SerialPort> },
}

pub struct SerialSource {
    mode: SourceMode,
}

impl SerialSource {
    /// Create a simulated entropy source seeded from system nanosecond clock.
    /// Used for development when no MYOSA hardware is connected.
    pub fn simulated() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);

        let state = if nanos == 0 {
            0x5EED_DEAD_BEEF_CAFE
        } else {
            nanos
        };

        Self {
            mode: SourceMode::Simulated { state },
        }
    }

    /// Create a hardware entropy source from a serial port path.
    ///
    /// `port_path` is the OS device path, e.g.:
    ///   - macOS: `/dev/tty.usbserial-1420` or `/dev/cu.usbmodem14201`
    ///   - Linux: `/dev/ttyUSB0` or `/dev/ttyACM0`
    ///   - Windows: `COM3`
    pub fn hardware(port_path: &str) -> Result<Self, EntropyError> {
        Self::hardware_with_baud(port_path, DEFAULT_BAUD_RATE)
    }

    /// Create a hardware entropy source with a custom baud rate.
    pub fn hardware_with_baud(port_path: &str, baud_rate: u32) -> Result<Self, EntropyError> {
        let port = serialport::new(port_path, baud_rate)
            .timeout(SERIAL_TIMEOUT)
            .open()
            .map_err(|e| EntropyError::SerialError {
                reason: format!("failed to open {}: {}", port_path, e),
            })?;

        log::info!("opened hardware entropy source: {} @ {} baud", port_path, baud_rate);

        Ok(Self {
            mode: SourceMode::Hardware { port },
        })
    }

    /// List available serial ports on the system.
    pub fn list_ports() -> Vec<String> {
        serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    pub fn is_simulated(&self) -> bool {
        matches!(self.mode, SourceMode::Simulated { .. })
    }

    /// Read raw bytes from the entropy source.
    pub fn read_raw(&mut self, length: usize) -> Result<Vec<u8>, EntropyError> {
        if length > MAX_READ_LEN {
            return Err(EntropyError::InvalidLength {
                requested: length,
                max: MAX_READ_LEN,
            });
        }

        match &mut self.mode {
            SourceMode::Simulated { state } => Ok(Self::simulate_bytes(state, length)),
            SourceMode::Hardware { port } => {
                let mut buf = vec![0u8; length];
                let mut total_read = 0;

                while total_read < length {
                    match port.read(&mut buf[total_read..]) {
                        Ok(0) => {
                            return Err(EntropyError::SerialError {
                                reason: format!(
                                    "serial EOF after {} of {} bytes",
                                    total_read, length
                                ),
                            });
                        }
                        Ok(n) => total_read += n,
                        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            if total_read == 0 {
                                return Err(EntropyError::SerialError {
                                    reason: "serial read timed out — is the hardware sending data?"
                                        .into(),
                                });
                            }
                            // Partial read is OK — return what we have.
                            buf.truncate(total_read);
                            return Ok(buf);
                        }
                        Err(e) => {
                            return Err(EntropyError::SerialError {
                                reason: format!("serial read error: {}", e),
                            });
                        }
                    }
                }

                Ok(buf)
            }
        }
    }

    /// Read and parse a full entropy packet from the source.
    /// In simulated mode, constructs a valid packet from simulated bytes.
    pub fn read_packet(&mut self) -> Result<EntropyPacket, EntropyError> {
        match &mut self.mode {
            SourceMode::Simulated { state } => {
                let payload = Self::simulate_bytes(state, 64);
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                let raw = PacketParser::build_raw(0x01, timestamp, &payload);
                PacketParser::parse(&raw)
            }
            SourceMode::Hardware { port } => {
                // Read header first (12 bytes), then payload + checksum.
                let mut header = [0u8; 12];
                port.read_exact(&mut header).map_err(|e| {
                    EntropyError::SerialError {
                        reason: format!("failed to read packet header: {}", e),
                    }
                })?;

                // Extract payload length from header bytes 10..12.
                let payload_len =
                    u16::from_le_bytes([header[10], header[11]]) as usize;

                // Read payload + 2 checksum bytes.
                let remaining = payload_len + 2;
                let mut tail = vec![0u8; remaining];
                port.read_exact(&mut tail).map_err(|e| {
                    EntropyError::SerialError {
                        reason: format!("failed to read packet payload: {}", e),
                    }
                })?;

                let mut full_packet = Vec::with_capacity(12 + remaining);
                full_packet.extend_from_slice(&header);
                full_packet.extend_from_slice(&tail);

                PacketParser::parse(&full_packet)
            }
        }
    }

    /// Simulated entropy via xorshift64 — NOT cryptographically secure.
    fn simulate_bytes(state: &mut u64, length: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(length);
        for _ in 0..length {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            out.push(*state as u8);
        }
        out
    }
}
