use serde::{Deserialize, Serialize};

use crate::quality::{QualityTier, SourceProfile};

/// Raw entropy packet parsed from UART serial data.
#[derive(Debug, Clone)]
pub struct EntropyPacket {
    pub source_id: u8,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    pub checksum: u16,
}

/// Aggregate statistics exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyStats {
    pub pool_fills: u64,
    pub bytes_generated: u64,
    pub reseed_count: u64,
    pub health_status: String,
    pub drbg_bytes_since_reseed: u64,
    /// Estimated weighted entropy accumulated in the pool (bits).
    pub pool_entropy_bits: f64,
    /// Whether the pool has enough weighted entropy to be well-seeded.
    pub pool_well_seeded: bool,
    /// Per-source quality summaries.
    pub source_quality: Vec<SourceQualitySummary>,
}

/// Compact per-source quality summary for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceQualitySummary {
    pub source_id: u8,
    pub tier: String,
    pub min_entropy_bits_per_byte: f64,
    pub confidence: f64,
    pub observations: u64,
    pub total_bytes: u64,
}

impl From<&SourceProfile> for SourceQualitySummary {
    fn from(p: &SourceProfile) -> Self {
        Self {
            source_id: p.source_id,
            tier: match p.tier {
                QualityTier::Excellent => "excellent",
                QualityTier::Adequate => "adequate",
                QualityTier::Degraded => "degraded",
                QualityTier::Failed => "failed",
                QualityTier::Unknown => "unknown",
            }
            .to_string(),
            min_entropy_bits_per_byte: p.min_entropy_estimate,
            confidence: p.confidence,
            observations: p.observations,
            total_bytes: p.total_bytes,
        }
    }
}

/// Configuration for password generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
        }
    }
}

impl PasswordConfig {
    /// Build the character set from the enabled categories.
    pub fn charset(&self) -> Vec<u8> {
        let mut set = Vec::new();
        if self.lowercase {
            set.extend(b'a'..=b'z');
        }
        if self.uppercase {
            set.extend(b'A'..=b'Z');
        }
        if self.digits {
            set.extend(b'0'..=b'9');
        }
        if self.symbols {
            set.extend(b"!@#$%^&*()-_=+[]{}<>?/~".iter());
        }
        set
    }
}
