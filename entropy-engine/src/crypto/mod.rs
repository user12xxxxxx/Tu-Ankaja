use crate::errors::EntropyError;
use crate::models::PasswordConfig;

pub struct CryptoOutput;

impl CryptoOutput {
    /// Generate a password from random bytes using rejection sampling.
    ///
    /// Rejection sampling avoids modulo bias: for a charset of size N,
    /// we only accept random bytes where `byte < largest_multiple_of_N_below_256`.
    /// Rejected bytes are discarded, not reused.
    pub fn generate_password(
        random_bytes: &[u8],
        config: &PasswordConfig,
    ) -> Result<String, EntropyError> {
        let charset = config.charset();
        if charset.is_empty() {
            return Err(EntropyError::InvalidLength {
                requested: config.length,
                max: 0,
            });
        }

        let charset_len = charset.len() as u16;
        // Largest multiple of charset_len that fits in a u8.
        let limit = (256 / charset_len) * charset_len;

        let mut password = Vec::with_capacity(config.length);

        for &byte in random_bytes {
            if password.len() >= config.length {
                break;
            }
            // Rejection sampling: discard bytes that would cause modulo bias.
            if u16::from(byte) < limit {
                let idx = (byte as usize) % charset.len();
                password.push(charset[idx]);
            }
        }

        if password.len() < config.length {
            return Err(EntropyError::InvalidLength {
                requested: config.length,
                max: password.len(),
            });
        }

        Ok(String::from_utf8(password).expect("charset is ASCII"))
    }

    /// Extract a 32-byte AES-256 key from random bytes.
    pub fn generate_aes256_key(random_bytes: &[u8]) -> Result<[u8; 32], EntropyError> {
        if random_bytes.len() < 32 {
            return Err(EntropyError::InvalidLength {
                requested: 32,
                max: random_bytes.len(),
            });
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&random_bytes[..32]);
        Ok(key)
    }

    /// Generate a hex-encoded session token from random bytes.
    pub fn generate_session_token(random_bytes: &[u8]) -> String {
        Self::to_hex(random_bytes)
    }

    pub fn to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_correct_length() {
        // Use bytes that will all pass rejection sampling for default charset (62+23=85 chars).
        let bytes: Vec<u8> = (0..200).collect();
        let config = PasswordConfig {
            length: 20,
            ..Default::default()
        };
        let pw = CryptoOutput::generate_password(&bytes, &config).unwrap();
        assert_eq!(pw.len(), 20);
    }

    #[test]
    fn password_uses_only_charset() {
        let bytes: Vec<u8> = (0..200).collect();
        let config = PasswordConfig {
            length: 50,
            uppercase: false,
            lowercase: true,
            digits: false,
            symbols: false,
        };
        let pw = CryptoOutput::generate_password(&bytes, &config).unwrap();
        assert!(pw.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn aes_key_is_32_bytes() {
        let bytes = [0xAB; 32];
        let key = CryptoOutput::generate_aes256_key(&bytes).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn session_token_is_hex() {
        let token = CryptoOutput::generate_session_token(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(token, "deadbeef");
    }

    #[test]
    fn rejects_empty_charset() {
        let config = PasswordConfig {
            length: 10,
            uppercase: false,
            lowercase: false,
            digits: false,
            symbols: false,
        };
        assert!(CryptoOutput::generate_password(&[0; 100], &config).is_err());
    }
}
