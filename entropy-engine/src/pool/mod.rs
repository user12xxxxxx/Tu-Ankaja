use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::quality::SourceQualityTracker;

/// Entropy accumulation pool using SHA-256 with source quality weighting.
///
/// Internal state is updated by hashing: new_state = SHA-256(state || input).
/// This ensures that entropy only accumulates — mixing in low-quality data
/// cannot reduce the entropy already in the pool.
///
/// The pool tracks which sources contributed and their estimated weighted
/// entropy contribution. This allows the system to know whether the pool
/// contains "enough" real entropy or is mostly low-quality padding.
///
/// The raw state is never exposed. `derive_seed()` uses domain separation
/// to produce a DRBG seed without revealing the pool's internal state.
pub struct EntropyPool {
    state: [u8; 32],
    fill_count: u64,
    quality_tracker: SourceQualityTracker,
    /// Accumulated weighted entropy estimate in bits.
    weighted_entropy_bits: f64,
}

/// Domain separation prefix for DRBG seed derivation.
const DRBG_SEED_DOMAIN: &[u8] = b"entropy-vault:drbg-seed:";

/// Minimum weighted entropy (bits) the pool should accumulate before
/// we consider the DRBG well-seeded.
const MIN_POOL_ENTROPY_BITS: f64 = 256.0;

/// Cap on weighted entropy bits to prevent unbounded growth.
/// Even with excellent sources, we cap at ~512 bits to prevent
/// long-running pools from claiming arbitrarily high entropy.
const MAX_POOL_ENTROPY_BITS: f64 = 512.0;

/// Each time entropy is consumed (via `derive_seed()`), decay the
/// pool estimate by this factor to reflect that entropy has been extracted.
const ENTROPY_CONSUMPTION_DECAY: f64 = 0.75;

impl EntropyPool {
    pub fn new() -> Self {
        Self {
            state: [0u8; 32],
            fill_count: 0,
            quality_tracker: SourceQualityTracker::new(),
            weighted_entropy_bits: 0.0,
        }
    }

    pub fn from_seed(seed: &[u8]) -> Self {
        let mut pool = Self::new();
        pool.mix(seed);
        pool
    }

    /// Mix new entropy into the pool without source tracking.
    ///
    /// Used for initial seeding or when source identity is unavailable.
    /// Does not contribute to weighted entropy estimates.
    pub fn mix(&mut self, input: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(&self.state);
        hasher.update(input);
        self.state = hasher.finalize().into();
        self.fill_count += 1;
    }

    /// Mix entropy from a known source, updating quality tracking.
    ///
    /// The source's quality profile is updated with this sample, and its
    /// weighted entropy contribution is accumulated in the pool's estimate.
    pub fn mix_from_source(&mut self, source_id: u8, input: &[u8]) {
        // Update source quality before computing contribution.
        self.quality_tracker.observe(source_id, input);

        // Compute this sample's weighted entropy contribution.
        let contribution = self
            .quality_tracker
            .weighted_entropy_bits(source_id, input.len());
        self.weighted_entropy_bits =
            (self.weighted_entropy_bits + contribution).min(MAX_POOL_ENTROPY_BITS);

        // Mix into pool state (always — even low-quality data can't reduce entropy).
        self.mix(input);
    }

    /// Derive a 32-byte seed for the DRBG using domain separation.
    ///
    /// Returns SHA-256(domain || state) — the raw state is never exposed,
    /// so compromising the DRBG seed does not reveal the pool state.
    /// Derive a 32-byte seed for the DRBG using domain separation.
    ///
    /// Returns SHA-256(domain || state) — the raw state is never exposed,
    /// so compromising the DRBG seed does not reveal the pool state.
    ///
    /// Decays the entropy estimate after extraction, reflecting that the
    /// pool's unique entropy has been partially consumed.
    pub fn derive_seed(&mut self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(DRBG_SEED_DOMAIN);
        hasher.update(&self.state);

        // Decay entropy estimate — extraction reduces the pool's advantage
        // over an adversary who might have observed the seed.
        self.weighted_entropy_bits *= ENTROPY_CONSUMPTION_DECAY;

        hasher.finalize().into()
    }

    /// Whether the pool has accumulated enough weighted entropy to be
    /// considered well-seeded (at least 256 bits of estimated real entropy).
    pub fn is_well_seeded(&self) -> bool {
        self.weighted_entropy_bits >= MIN_POOL_ENTROPY_BITS
    }

    /// Current weighted entropy estimate in bits.
    pub fn weighted_entropy_bits(&self) -> f64 {
        self.weighted_entropy_bits
    }

    pub fn fill_count(&self) -> u64 {
        self.fill_count
    }

    pub fn quality_tracker(&self) -> &SourceQualityTracker {
        &self.quality_tracker
    }
}

impl Default for EntropyPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EntropyPool {
    fn drop(&mut self) {
        self.state.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mix_changes_state() {
        let mut pool = EntropyPool::new();
        let before = pool.derive_seed();
        pool.mix(b"entropy input");
        assert_ne!(before, pool.derive_seed());
    }

    #[test]
    fn deterministic_for_same_input() {
        let mut a = EntropyPool::new();
        let mut b = EntropyPool::new();
        a.mix(b"same");
        b.mix(b"same");
        assert_eq!(a.derive_seed(), b.derive_seed());
    }

    #[test]
    fn different_inputs_differ() {
        let mut a = EntropyPool::from_seed(b"aaa");
        let mut b = EntropyPool::from_seed(b"bbb");
        a.mix(b"x");
        b.mix(b"x");
        assert_ne!(a.derive_seed(), b.derive_seed());
    }

    #[test]
    fn derive_seed_is_not_raw_state() {
        let mut pool = EntropyPool::new();
        pool.mix(b"test entropy");
        let seed = pool.derive_seed();
        let mut hasher = Sha256::new();
        hasher.update([0u8; 32]);
        hasher.update(b"test entropy");
        let raw_state: [u8; 32] = hasher.finalize().into();
        assert_ne!(seed, raw_state, "derive_seed must not equal raw state");
    }

    #[test]
    fn source_aware_mix_tracks_entropy() {
        let mut pool = EntropyPool::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();

        assert_eq!(pool.weighted_entropy_bits(), 0.0);

        // First observations build quality profile.
        for _ in 0..15 {
            pool.mix_from_source(0x01, &good);
        }

        assert!(
            pool.weighted_entropy_bits() > 0.0,
            "should have accumulated entropy: got {}",
            pool.weighted_entropy_bits()
        );
    }

    #[test]
    fn low_quality_source_contributes_less() {
        let mut pool_good = EntropyPool::new();
        let mut pool_bad = EntropyPool::new();

        let good: Vec<u8> = (0..=255).cycle().take(512).collect();
        let bad = vec![0x00; 512];

        for _ in 0..20 {
            pool_good.mix_from_source(0x01, &good);
            pool_bad.mix_from_source(0x01, &bad);
        }

        assert!(
            pool_good.weighted_entropy_bits() > pool_bad.weighted_entropy_bits(),
            "good source should contribute more entropy: good={}, bad={}",
            pool_good.weighted_entropy_bits(),
            pool_bad.weighted_entropy_bits()
        );
    }

    #[test]
    fn well_seeded_requires_sufficient_entropy() {
        let mut pool = EntropyPool::new();
        assert!(!pool.is_well_seeded());

        let good: Vec<u8> = (0..=255).cycle().take(512).collect();
        for _ in 0..30 {
            pool.mix_from_source(0x01, &good);
        }

        assert!(
            pool.is_well_seeded(),
            "pool should be well-seeded after many good samples: {} bits",
            pool.weighted_entropy_bits()
        );
    }

    #[test]
    fn entropy_bits_capped_at_maximum() {
        let mut pool = EntropyPool::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();

        // Feed an excessive number of high-quality samples.
        for _ in 0..200 {
            pool.mix_from_source(0x01, &good);
        }

        assert!(
            pool.weighted_entropy_bits() <= super::MAX_POOL_ENTROPY_BITS,
            "pool entropy must be capped: got {} bits",
            pool.weighted_entropy_bits()
        );
    }

    #[test]
    fn derive_seed_decays_entropy_estimate() {
        let mut pool = EntropyPool::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();

        for _ in 0..30 {
            pool.mix_from_source(0x01, &good);
        }
        let before = pool.weighted_entropy_bits();
        assert!(before > 0.0);

        let _seed = pool.derive_seed();
        let after = pool.weighted_entropy_bits();

        assert!(
            after < before,
            "entropy should decay after seed derivation: before={before}, after={after}"
        );
    }

    #[test]
    fn empty_input_does_not_panic() {
        let mut pool = EntropyPool::new();
        pool.mix(b"");
        pool.mix_from_source(0x01, b"");
        let _seed = pool.derive_seed();
    }
}
