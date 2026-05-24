use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// SHA-256 entropy conditioning (whitening).
///
/// Transforms raw hardware entropy into uniformly distributed bytes
/// by applying SHA-256. This removes statistical biases present in
/// raw ADC / sensor readings.
pub struct Whitener;

impl Whitener {
    /// Condition input into a single 32-byte SHA-256 digest.
    pub fn condition_fixed(input: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.finalize().into()
    }

    /// Condition input into variable-length output.
    ///
    /// Produces chained SHA-256 blocks:
    ///   block_0 = SHA-256(input || counter_bytes)
    ///   block_n = SHA-256(block_{n-1} || input || counter_bytes)
    ///
    /// Uses a u32 counter (M1 fix: supports up to ~137 GB before wrap,
    /// vs u8 which wrapped at 8 KB).
    pub fn condition(input: &[u8], output_len: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(output_len);
        let mut prev_block = [0u8; 32];

        let mut counter: u32 = 0;
        while result.len() < output_len {
            let mut hasher = Sha256::new();
            if counter > 0 {
                hasher.update(prev_block);
            }
            hasher.update(input);
            hasher.update(counter.to_le_bytes());
            prev_block = hasher.finalize().into();
            result.extend_from_slice(&prev_block);
            counter = counter.wrapping_add(1);
        }

        // Z1 fix: zeroize intermediate digest.
        prev_block.zeroize();

        result.truncate(output_len);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_output_is_32_bytes() {
        let out = Whitener::condition_fixed(b"test input");
        assert_eq!(out.len(), 32);
    }

    #[test]
    fn output_differs_from_input() {
        let input = b"raw hardware entropy bytes here!";
        let out = Whitener::condition_fixed(input);
        assert_ne!(&out[..], &input[..]);
    }

    #[test]
    fn deterministic() {
        let a = Whitener::condition_fixed(b"same");
        let b = Whitener::condition_fixed(b"same");
        assert_eq!(a, b);
    }

    #[test]
    fn variable_length_output() {
        let out = Whitener::condition(b"data", 100);
        assert_eq!(out.len(), 100);

        let short = Whitener::condition(b"data", 16);
        assert_eq!(short.len(), 16);
    }

    #[test]
    fn different_inputs_differ() {
        let a = Whitener::condition_fixed(b"input_a");
        let b = Whitener::condition_fixed(b"input_b");
        assert_ne!(a, b);
    }

    #[test]
    fn large_output_beyond_old_u8_limit() {
        // M1 regression test: output > 8192 bytes should not have repeated blocks.
        let out = Whitener::condition(b"test", 8224); // 257 blocks × 32
        let block_256 = &out[8192 - 32..8192]; // block at index 255
        let block_257 = &out[8192..8224]; // block at index 256
        assert_ne!(
            block_256, block_257,
            "blocks must not repeat after u8 range"
        );
    }
}
