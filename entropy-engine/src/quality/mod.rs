use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimum observations before a source's entropy estimate is trusted.
const MIN_OBSERVATIONS: u64 = 10;

/// EWMA smoothing factor for entropy estimates (0..1).
const EWMA_ALPHA: f64 = 0.15;

/// Below this min-entropy (bits/byte), the source is considered degraded.
const DEGRADATION_THRESHOLD: f64 = 4.0;

/// Below this min-entropy, the source is considered failed/stuck.
const FAILURE_THRESHOLD: f64 = 1.0;

/// Minimum observations per source pair before correlation is evaluated.
const MIN_CORRELATION_SAMPLES: usize = 10;

/// Pearson |r| above this = high correlation → heavy discount.
const HIGH_CORRELATION_THRESHOLD: f64 = 0.7;

/// Pearson |r| above this = moderate correlation → partial discount.
const MODERATE_CORRELATION_THRESHOLD: f64 = 0.4;

/// Per-source quality profile tracking entropy contribution confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProfile {
    pub source_id: u8,
    pub observations: u64,
    pub min_entropy_estimate: f64,
    pub confidence: f64,
    pub tier: QualityTier,
    pub total_bytes: u64,
    pub consecutive_degraded: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityTier {
    Excellent,
    Adequate,
    Degraded,
    Failed,
    Unknown,
}

impl SourceProfile {
    fn new(source_id: u8) -> Self {
        Self {
            source_id,
            observations: 0,
            min_entropy_estimate: 8.0,
            confidence: 0.0,
            tier: QualityTier::Unknown,
            total_bytes: 0,
            consecutive_degraded: 0,
        }
    }
}

/// Rolling byte-value distribution fingerprint for a single source.
/// Used to detect correlation between sources.
#[derive(Debug, Clone)]
struct DistributionFingerprint {
    /// Cumulative byte counts (smoothed via EWMA on normalized form).
    counts: [f64; 256],
    /// Number of distributions merged in.
    samples: usize,
}

impl DistributionFingerprint {
    fn new() -> Self {
        Self {
            counts: [0.0; 256],
            samples: 0,
        }
    }

    /// Update the fingerprint with a new sample's byte distribution.
    fn update(&mut self, sample: &[u8]) {
        if sample.is_empty() {
            return;
        }

        // Compute normalized distribution of this sample.
        let mut raw = [0u32; 256];
        for &byte in sample {
            raw[byte as usize] += 1;
        }

        let len = sample.len() as f64;
        if self.samples == 0 {
            for i in 0..256 {
                self.counts[i] = raw[i] as f64 / len;
            }
        } else {
            // EWMA blend with existing fingerprint.
            let alpha = 0.2;
            for i in 0..256 {
                let new_val = raw[i] as f64 / len;
                self.counts[i] = alpha * new_val + (1.0 - alpha) * self.counts[i];
            }
        }
        self.samples += 1;
    }
}

/// Pairwise correlation record between two sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub source_a: u8,
    pub source_b: u8,
    /// Pearson correlation coefficient between distributions (-1.0 to 1.0).
    pub correlation: f64,
    /// Discount factor applied to the lower-confidence source (0.0 to 1.0).
    /// 1.0 = no discount (independent), 0.0 = fully discounted (identical).
    pub independence_factor: f64,
}

/// Tracks quality of all entropy sources, detects inter-source correlation,
/// and computes weighted entropy contribution estimates.
pub struct SourceQualityTracker {
    sources: HashMap<u8, SourceProfile>,
    fingerprints: HashMap<u8, DistributionFingerprint>,
    /// Cached pairwise correlations, updated on each observation.
    correlations: Vec<CorrelationPair>,
}

impl SourceQualityTracker {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            fingerprints: HashMap::new(),
            correlations: Vec::new(),
        }
    }

    /// Observe a sample from a specific source and update quality + correlation.
    pub fn observe(&mut self, source_id: u8, sample: &[u8]) {
        // Update source quality profile.
        let profile = self
            .sources
            .entry(source_id)
            .or_insert_with(|| SourceProfile::new(source_id));

        profile.observations += 1;
        profile.total_bytes += sample.len() as u64;

        let sample_entropy = Self::estimate_min_entropy(sample);

        if profile.observations == 1 {
            profile.min_entropy_estimate = sample_entropy;
        } else {
            profile.min_entropy_estimate =
                EWMA_ALPHA * sample_entropy + (1.0 - EWMA_ALPHA) * profile.min_entropy_estimate;
        }

        profile.tier = if profile.observations < MIN_OBSERVATIONS {
            QualityTier::Unknown
        } else if profile.min_entropy_estimate >= 6.0 {
            QualityTier::Excellent
        } else if profile.min_entropy_estimate >= DEGRADATION_THRESHOLD {
            QualityTier::Adequate
        } else if profile.min_entropy_estimate >= FAILURE_THRESHOLD {
            QualityTier::Degraded
        } else {
            QualityTier::Failed
        };

        if sample_entropy < DEGRADATION_THRESHOLD {
            profile.consecutive_degraded += 1;
        } else {
            profile.consecutive_degraded = 0;
        }

        Self::update_confidence(profile);

        // Update distribution fingerprint for correlation detection.
        let fp = self
            .fingerprints
            .entry(source_id)
            .or_insert_with(DistributionFingerprint::new);
        fp.update(sample);

        // Recompute pairwise correlations if we have multiple sources.
        if self.fingerprints.len() > 1 {
            self.recompute_correlations();
        }
    }

    pub fn get_profile(&self, source_id: u8) -> Option<&SourceProfile> {
        self.sources.get(&source_id)
    }

    pub fn all_profiles(&self) -> Vec<&SourceProfile> {
        self.sources.values().collect()
    }

    /// Get all detected pairwise correlations.
    pub fn correlations(&self) -> &[CorrelationPair] {
        &self.correlations
    }

    /// Compute weighted entropy contribution, applying correlation discounts.
    ///
    /// If this source is correlated with another source that has already
    /// contributed to the pool, its contribution is discounted by the
    /// independence factor to avoid overestimating total entropy.
    pub fn weighted_entropy_bits(&self, source_id: u8, sample_bytes: usize) -> f64 {
        let profile = match self.sources.get(&source_id) {
            Some(p) => p,
            None => return 0.0,
        };

        let raw_contribution =
            sample_bytes as f64 * profile.min_entropy_estimate * profile.confidence;

        // Apply the worst (lowest) independence factor from any correlated source.
        let independence = self.independence_factor_for(source_id);

        raw_contribution * independence
    }

    /// Get the effective independence factor for a source.
    ///
    /// For each correlated pair, the source with higher confidence (or lower
    /// source_id as tiebreaker) is the "primary" and keeps full contribution.
    /// The secondary source is discounted by the independence factor.
    /// This prevents double-counting while preserving entropy from the best source.
    fn independence_factor_for(&self, source_id: u8) -> f64 {
        let mut min_factor = 1.0_f64;

        for pair in &self.correlations {
            if pair.source_a == source_id || pair.source_b == source_id {
                let partner = if pair.source_a == source_id {
                    pair.source_b
                } else {
                    pair.source_a
                };

                if let Some(partner_profile) = self.sources.get(&partner) {
                    if partner_profile.confidence <= 0.0 {
                        continue;
                    }

                    // Determine if this source is the secondary (discounted) one.
                    let my_profile = match self.sources.get(&source_id) {
                        Some(p) => p,
                        None => continue,
                    };

                    // Primary = higher confidence, or lower source_id as tiebreaker.
                    let i_am_secondary = partner_profile.confidence > my_profile.confidence
                        || (partner_profile.confidence == my_profile.confidence
                            && source_id > partner);

                    if i_am_secondary {
                        min_factor = min_factor.min(pair.independence_factor);
                    }
                }
            }
        }

        min_factor
    }

    /// Recompute all pairwise correlations from current fingerprints.
    fn recompute_correlations(&mut self) {
        self.correlations.clear();

        let ids: Vec<u8> = self.fingerprints.keys().copied().collect();

        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let a_id = ids[i];
                let b_id = ids[j];

                let fp_a = &self.fingerprints[&a_id];
                let fp_b = &self.fingerprints[&b_id];

                // Only evaluate if both have enough samples.
                if fp_a.samples < MIN_CORRELATION_SAMPLES || fp_b.samples < MIN_CORRELATION_SAMPLES
                {
                    continue;
                }

                let r = Self::pearson_correlation(&fp_a.counts, &fp_b.counts);
                let abs_r = r.abs();

                let independence_factor = if abs_r >= HIGH_CORRELATION_THRESHOLD {
                    // Highly correlated: discount the weaker source heavily.
                    // At |r|=1.0 → factor=0.0, at |r|=0.7 → factor≈0.3
                    1.0 - abs_r
                } else if abs_r >= MODERATE_CORRELATION_THRESHOLD {
                    // Moderate correlation: partial discount.
                    // Linear interpolation: at 0.4 → ~0.8, at 0.7 → ~0.3
                    1.0 - (abs_r - MODERATE_CORRELATION_THRESHOLD)
                        / (1.0 - MODERATE_CORRELATION_THRESHOLD)
                        * abs_r
                } else {
                    // Low/no correlation: fully independent.
                    1.0
                };

                self.correlations.push(CorrelationPair {
                    source_a: a_id,
                    source_b: b_id,
                    correlation: r,
                    independence_factor,
                });
            }
        }
    }

    /// Pearson correlation coefficient between two 256-element distributions.
    fn pearson_correlation(a: &[f64; 256], b: &[f64; 256]) -> f64 {
        let n = 256.0;

        let mean_a: f64 = a.iter().sum::<f64>() / n;
        let mean_b: f64 = b.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_a = 0.0;
        let mut var_b = 0.0;

        for i in 0..256 {
            let da = a[i] - mean_a;
            let db = b[i] - mean_b;
            cov += da * db;
            var_a += da * da;
            var_b += db * db;
        }

        let denom = (var_a * var_b).sqrt();
        if denom < 1e-15 {
            // Both distributions are nearly flat (e.g. uniform).
            // Pearson is undefined, but if the distributions are identical,
            // the sources are producing the same output → correlated.
            let l2_sq: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum();
            return if l2_sq < 1e-10 { 1.0 } else { 0.0 };
        }

        cov / denom
    }

    /// Estimate min-entropy using Most Common Value estimator (SP 800-90B §6.3.1).
    pub(crate) fn estimate_min_entropy(sample: &[u8]) -> f64 {
        if sample.is_empty() {
            return 0.0;
        }

        let mut counts = [0u32; 256];
        for &byte in sample {
            counts[byte as usize] += 1;
        }

        let max_count = counts.iter().copied().max().unwrap_or(0);
        let p_max = max_count as f64 / sample.len() as f64;

        if p_max <= 0.0 || p_max >= 1.0 {
            return if p_max >= 1.0 { 0.0 } else { 8.0 };
        }

        -p_max.log2()
    }

    fn update_confidence(profile: &mut SourceProfile) {
        let target = match profile.tier {
            QualityTier::Unknown => 0.1,
            QualityTier::Excellent => 1.0,
            QualityTier::Adequate => 0.7,
            QualityTier::Degraded => 0.3,
            QualityTier::Failed => 0.0,
        };

        let degradation_penalty = if profile.consecutive_degraded > 5 {
            0.5_f64.min(profile.consecutive_degraded as f64 * 0.05)
        } else {
            0.0
        };

        let adjusted_target = (target - degradation_penalty).max(0.0);

        if adjusted_target < profile.confidence {
            profile.confidence += 0.3 * (adjusted_target - profile.confidence);
        } else {
            profile.confidence += 0.1 * (adjusted_target - profile.confidence);
        }

        profile.confidence = profile.confidence.clamp(0.0, 1.0);
    }
}

impl Default for SourceQualityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_sample_has_high_entropy() {
        let sample: Vec<u8> = (0..=255).cycle().take(1024).collect();
        let h = SourceQualityTracker::estimate_min_entropy(&sample);
        assert!(h > 7.5, "uniform distribution should have ~8 bits: got {h}");
    }

    #[test]
    fn constant_sample_has_zero_entropy() {
        let sample = vec![0xAA; 256];
        let h = SourceQualityTracker::estimate_min_entropy(&sample);
        assert!(h < 0.01, "constant sample should have ~0 bits: got {h}");
    }

    #[test]
    fn biased_sample_has_reduced_entropy() {
        let mut sample = vec![0x00; 192];
        sample.extend(vec![0xFF; 64]);
        let h = SourceQualityTracker::estimate_min_entropy(&sample);
        assert!(h > 0.3 && h < 0.6, "biased sample entropy: got {h}");
    }

    #[test]
    fn source_builds_confidence_over_observations() {
        let mut tracker = SourceQualityTracker::new();
        let good_sample: Vec<u8> = (0..=255).cycle().take(512).collect();

        for _ in 0..5 {
            tracker.observe(0x01, &good_sample);
        }
        let profile = tracker.get_profile(0x01).unwrap();
        assert_eq!(profile.tier, QualityTier::Unknown);
        assert!(profile.confidence < 0.5);

        for _ in 0..20 {
            tracker.observe(0x01, &good_sample);
        }
        let profile = tracker.get_profile(0x01).unwrap();
        assert_eq!(profile.tier, QualityTier::Excellent);
        assert!(profile.confidence > 0.5);
    }

    #[test]
    fn degraded_source_loses_confidence() {
        let mut tracker = SourceQualityTracker::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();

        for _ in 0..30 {
            tracker.observe(0x02, &good);
        }
        let before = tracker.get_profile(0x02).unwrap().confidence;

        let bad = vec![0x00; 512];
        for _ in 0..20 {
            tracker.observe(0x02, &bad);
        }
        let after = tracker.get_profile(0x02).unwrap().confidence;
        assert!(after < before);
    }

    #[test]
    fn weighted_entropy_scales_with_confidence() {
        let mut tracker = SourceQualityTracker::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();

        tracker.observe(0x03, &good);
        let w1 = tracker.weighted_entropy_bits(0x03, 64);

        for _ in 0..30 {
            tracker.observe(0x03, &good);
        }
        let w2 = tracker.weighted_entropy_bits(0x03, 64);
        assert!(w2 > w1);
    }

    #[test]
    fn multiple_sources_tracked_independently() {
        let mut tracker = SourceQualityTracker::new();
        let good: Vec<u8> = (0..=255).cycle().take(512).collect();
        let bad = vec![0x00; 512];

        for _ in 0..15 {
            tracker.observe(0x01, &good);
            tracker.observe(0x02, &bad);
        }

        let p1 = tracker.get_profile(0x01).unwrap();
        let p2 = tracker.get_profile(0x02).unwrap();
        assert_eq!(p1.tier, QualityTier::Excellent);
        assert_eq!(p2.tier, QualityTier::Failed);
    }

    // --- Correlation detection tests ---

    #[test]
    fn identical_sources_are_highly_correlated() {
        let mut tracker = SourceQualityTracker::new();
        // Two sources producing the exact same distribution.
        let data: Vec<u8> = (0..=255).cycle().take(512).collect();

        for _ in 0..15 {
            tracker.observe(0x01, &data);
            tracker.observe(0x02, &data);
        }

        let correlations = tracker.correlations();
        assert_eq!(correlations.len(), 1);
        let pair = &correlations[0];
        assert!(
            pair.correlation > 0.9,
            "identical distributions should correlate >0.9: got {}",
            pair.correlation
        );
        assert!(
            pair.independence_factor < 0.3,
            "correlated sources should have low independence: got {}",
            pair.independence_factor
        );
    }

    #[test]
    fn independent_sources_have_low_correlation() {
        let mut tracker = SourceQualityTracker::new();

        // Source A: uniform distribution.
        let data_a: Vec<u8> = (0..=255).cycle().take(512).collect();

        // Source B: heavily biased distribution (only low byte values).
        let data_b: Vec<u8> = (0..32).cycle().take(512).collect();

        for _ in 0..15 {
            tracker.observe(0x01, &data_a);
            tracker.observe(0x02, &data_b);
        }

        let correlations = tracker.correlations();
        assert_eq!(correlations.len(), 1);
        let pair = &correlations[0];
        // These distributions are very different, so correlation should be low.
        assert!(
            pair.independence_factor > 0.5,
            "different distributions should have higher independence: got {}",
            pair.independence_factor
        );
    }

    #[test]
    fn correlated_source_contributes_less_entropy() {
        let mut tracker = SourceQualityTracker::new();
        let same_data: Vec<u8> = (0..=255).cycle().take(512).collect();

        // Build up two correlated sources.
        for _ in 0..20 {
            tracker.observe(0x01, &same_data);
            tracker.observe(0x02, &same_data);
        }

        // With correlation, the second source should contribute less.
        let w_source1 = tracker.weighted_entropy_bits(0x01, 64);
        let w_source2 = tracker.weighted_entropy_bits(0x02, 64);

        // Source 0x01 is primary (lower ID, same confidence) → full contribution.
        // Source 0x02 is secondary → discounted by independence factor.
        assert!(w_source1 > 0.0, "primary source should contribute");
        assert!(
            w_source2 == 0.0 || w_source2 < w_source1,
            "secondary source should contribute less"
        );

        // Now compare source 0x02 (secondary) against a solo tracker.
        let mut solo_tracker = SourceQualityTracker::new();
        for _ in 0..20 {
            solo_tracker.observe(0x02, &same_data);
        }
        let w_solo = solo_tracker.weighted_entropy_bits(0x02, 64);

        // The secondary correlated source should contribute less than an independent one.
        assert!(
            w_source2 < w_solo,
            "correlated secondary source should contribute less: correlated={w_source2}, solo={w_solo}"
        );
    }

    #[test]
    fn no_correlation_with_single_source() {
        let mut tracker = SourceQualityTracker::new();
        let data: Vec<u8> = (0..=255).cycle().take(512).collect();

        for _ in 0..15 {
            tracker.observe(0x01, &data);
        }

        assert!(tracker.correlations().is_empty());
        assert!((tracker.independence_factor_for(0x01) - 1.0).abs() < f64::EPSILON);
    }
}
