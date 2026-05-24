use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::ChaCha20;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

use crate::errors::EntropyError;

/// Maximum bytes that can be generated before a reseed is mandatory.
const RESEED_INTERVAL: u64 = 1 << 20; // 1 MiB

/// ChaCha20-based deterministic random bit generator.
///
/// Uses a 256-bit key derived from the entropy pool. Each reseed increments
/// the generation counter (used as nonce). Between reseeds, an internal
/// stream position counter ensures successive `fill()` calls never repeat
/// keystream bytes.
///
/// After each `fill()`, the key is ratcheted forward: the next 32 bytes of
/// keystream replace the key, providing forward secrecy — compromising the
/// current state does not reveal past outputs.
pub struct Drbg {
    key: [u8; 32],
    generation: u64,
    stream_position: u64,
    bytes_since_reseed: u64,
}

impl Drbg {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            key: seed,
            generation: 0,
            stream_position: 0,
            bytes_since_reseed: 0,
        }
    }

    /// Mix new entropy into the DRBG key: key = SHA-256(key || entropy).
    /// Resets the stream position and advances the generation nonce.
    pub fn reseed(&mut self, entropy: [u8; 32]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.key);
        hasher.update(&entropy);
        self.key = hasher.finalize().into();
        self.bytes_since_reseed = 0;
        self.stream_position = 0;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Generate `length` pseudorandom bytes.
    ///
    /// Returns `Err(DrbgReseedRequired)` if the reseed interval would be exceeded.
    /// After generating output, the key is ratcheted forward for forward secrecy.
    pub fn fill(&mut self, length: usize) -> Result<Vec<u8>, EntropyError> {
        // Account for output bytes + 32-byte key ratchet.
        let total_needed = length as u64 + 32;
        if self.bytes_since_reseed.saturating_add(total_needed) > RESEED_INTERVAL {
            return Err(EntropyError::DrbgReseedRequired {
                bytes_generated: self.bytes_since_reseed,
            });
        }

        let mut nonce = [0u8; 12];
        nonce[..8].copy_from_slice(&self.generation.to_le_bytes());

        let mut cipher = ChaCha20::new((&self.key).into(), (&nonce).into());

        // Seek to current stream position so successive fills never overlap.
        cipher.seek(self.stream_position);

        // Generate the requested output.
        let mut output = vec![0u8; length];
        cipher.apply_keystream(&mut output);

        // Key ratchet: consume 32 more keystream bytes to derive the next key.
        // This provides forward secrecy — the old key is overwritten.
        let mut new_key = Zeroizing::new([0u8; 32]);
        cipher.apply_keystream(new_key.as_mut());
        self.key.zeroize();
        self.key = *new_key;

        self.stream_position += (length as u64) + 32;
        self.bytes_since_reseed += (length as u64) + 32;

        Ok(output)
    }

    pub fn needs_reseed(&self) -> bool {
        self.bytes_since_reseed >= RESEED_INTERVAL
    }

    pub fn bytes_since_reseed(&self) -> u64 {
        self.bytes_since_reseed
    }
}

impl Drop for Drbg {
    fn drop(&mut self) {
        self.key.zeroize();
        self.generation = 0;
        self.stream_position = 0;
        self.bytes_since_reseed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_requested_length() {
        let mut drbg = Drbg::from_seed([0xAB; 32]);
        let out = drbg.fill(64).unwrap();
        assert_eq!(out.len(), 64);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = Drbg::from_seed([0x42; 32]);
        let mut b = Drbg::from_seed([0x42; 32]);
        assert_eq!(a.fill(32).unwrap(), b.fill(32).unwrap());
    }

    #[test]
    fn successive_fills_differ_without_reseed() {
        // C1 fix: successive fill() calls must produce different output.
        let mut drbg = Drbg::from_seed([0x42; 32]);
        let first = drbg.fill(32).unwrap();
        let second = drbg.fill(32).unwrap();
        assert_ne!(first, second, "successive fills must not repeat keystream");
    }

    #[test]
    fn reseed_changes_output() {
        let mut a = Drbg::from_seed([0x42; 32]);
        let mut b = Drbg::from_seed([0x42; 32]);
        b.reseed([0xFF; 32]);
        assert_ne!(a.fill(32).unwrap(), b.fill(32).unwrap());
    }

    #[test]
    fn output_is_not_zeros() {
        let mut drbg = Drbg::from_seed([0x01; 32]);
        let out = drbg.fill(256).unwrap();
        assert!(out.iter().any(|&b| b != 0));
    }

    #[test]
    fn forward_secrecy_key_changes_after_fill() {
        let mut a = Drbg::from_seed([0x99; 32]);
        let mut b = Drbg::from_seed([0x99; 32]);
        let _ = a.fill(16).unwrap();
        let _ = b.fill(32).unwrap();
        let a2 = a.fill(32).unwrap();
        let b2 = b.fill(32).unwrap();
        assert_ne!(a2, b2);
    }

    #[test]
    fn reseed_interval_enforced() {
        let mut drbg = Drbg::from_seed([0x42; 32]);
        // RESEED_INTERVAL = 1 MiB. Each fill consumes length + 32 (key ratchet).
        // Fill in chunks just under the limit.
        let chunk = 8192;
        let mut total = 0u64;
        loop {
            let result = drbg.fill(chunk);
            if result.is_err() {
                break;
            }
            total += (chunk as u64) + 32; // account for key ratchet
        }
        // Should have generated close to 1 MiB before hitting the limit.
        assert!(
            total >= (1 << 20) - (chunk as u64 + 32) * 2,
            "should generate near 1 MiB before reseed required: got {total}"
        );
    }

    #[test]
    fn reseed_resets_byte_counter() {
        let mut drbg = Drbg::from_seed([0x42; 32]);
        let _ = drbg.fill(1024).unwrap();
        assert!(drbg.bytes_since_reseed() > 0);
        drbg.reseed([0xFF; 32]);
        assert_eq!(drbg.bytes_since_reseed(), 0);
    }

    #[test]
    fn zero_length_fill() {
        let mut drbg = Drbg::from_seed([0x42; 32]);
        let out = drbg.fill(0).unwrap();
        assert!(out.is_empty());
    }
}
