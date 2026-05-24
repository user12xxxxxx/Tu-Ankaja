use serde::Serialize;
use std::fmt;
use zeroize::Zeroize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Failed,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Warning => write!(f, "warning"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// SP 800-90B inspired health monitoring.
///
/// Runs three checks on each entropy sample:
/// 1. **Repetition count test** — per-byte repetition detection (§4.4.1).
/// 2. **Adaptive proportion test** — byte value dominance in a sliding window (§4.4.2).
/// 3. **Entropy degradation** — low byte diversity in the sample.
///
/// Recovery from `Failed` requires `RECOVERY_THRESHOLD` consecutive healthy
/// observations to prevent oscillation from a flaky source.
pub struct HealthMonitor {
    // Repetition count test state (per-byte, not per-sample)
    last_byte: Option<u8>,
    byte_repetition_count: usize,
    byte_repetition_cutoff: usize,
    sample_repetition_failed: bool,

    // Adaptive proportion test state
    proportion_window: Vec<u8>,
    proportion_window_size: usize,
    proportion_cutoff_pct: u8,

    // Recovery tracking
    consecutive_healthy: usize,

    // Overall
    total_observations: u64,
    current_status: HealthStatus,
}

/// Number of consecutive healthy observations required to recover from Failed.
const RECOVERY_THRESHOLD: usize = 5;

/// Maximum bytes allowed in the proportion window to prevent memory spikes.
const MAX_SAMPLE_SIZE: usize = 65536;

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            last_byte: None,
            byte_repetition_count: 0,
            byte_repetition_cutoff: 20, // 20 identical consecutive bytes = failure
            sample_repetition_failed: false,

            proportion_window: Vec::with_capacity(4096),
            proportion_window_size: 4096,
            proportion_cutoff_pct: 50,

            consecutive_healthy: 0,

            total_observations: 0,
            current_status: HealthStatus::Healthy,
        }
    }

    /// Feed a new entropy sample into the health monitor.
    pub fn observe(&mut self, sample: &[u8]) {
        self.total_observations += 1;

        let rep_ok = self.repetition_count_test(sample);
        let prop_ok = self.adaptive_proportion_test(sample);
        let degrade_ok = self.check_degradation(sample);

        let this_round_healthy = rep_ok && prop_ok && degrade_ok;
        let this_round_warning = rep_ok && prop_ok && !degrade_ok;

        match self.current_status {
            HealthStatus::Failed => {
                // H2 fix: require sustained healthy observations to recover.
                if this_round_healthy {
                    self.consecutive_healthy += 1;
                    if self.consecutive_healthy >= RECOVERY_THRESHOLD {
                        self.current_status = HealthStatus::Healthy;
                        self.consecutive_healthy = 0;
                    }
                } else {
                    self.consecutive_healthy = 0;
                    // Stay Failed.
                }
            }
            _ => {
                if !rep_ok || !prop_ok {
                    self.current_status = HealthStatus::Failed;
                    self.consecutive_healthy = 0;
                } else if this_round_warning {
                    self.current_status = HealthStatus::Warning;
                    self.consecutive_healthy = 0;
                } else {
                    self.current_status = HealthStatus::Healthy;
                }
            }
        }
    }

    pub fn status(&self) -> HealthStatus {
        self.current_status
    }

    pub fn is_healthy(&self) -> bool {
        self.current_status == HealthStatus::Healthy
    }

    /// Repetition count test (SP 800-90B §4.4.1) — per-byte granularity.
    ///
    /// Scans each byte in the sample. If any byte value repeats consecutively
    /// `byte_repetition_cutoff` times, the test fails.
    fn repetition_count_test(&mut self, sample: &[u8]) -> bool {
        self.sample_repetition_failed = false;

        for &byte in sample {
            if self.last_byte == Some(byte) {
                self.byte_repetition_count += 1;
                if self.byte_repetition_count >= self.byte_repetition_cutoff {
                    self.sample_repetition_failed = true;
                }
            } else {
                self.byte_repetition_count = 1;
            }
            self.last_byte = Some(byte);
        }

        !self.sample_repetition_failed
    }

    /// Adaptive proportion test (SP 800-90B §4.4.2).
    ///
    /// Maintains a sliding window of recent bytes. If any single byte value
    /// exceeds `proportion_cutoff_pct`% of the window, the test fails.
    fn adaptive_proportion_test(&mut self, sample: &[u8]) -> bool {
        // M4 fix: cap incoming sample size to prevent memory spike.
        let effective = if sample.len() > MAX_SAMPLE_SIZE {
            &sample[..MAX_SAMPLE_SIZE]
        } else {
            sample
        };

        self.proportion_window.extend_from_slice(effective);

        // Trim to window size (keep most recent bytes).
        if self.proportion_window.len() > self.proportion_window_size {
            let excess = self.proportion_window.len() - self.proportion_window_size;
            self.proportion_window.drain(..excess);
        }

        // Only evaluate once we have enough data.
        if self.proportion_window.len() < self.proportion_window_size / 2 {
            return true;
        }

        // Count frequency of each byte value.
        let mut counts = [0u32; 256];
        for &byte in &self.proportion_window {
            counts[byte as usize] += 1;
        }

        let max_count = counts.iter().copied().max().unwrap_or(0);
        let threshold =
            (self.proportion_window.len() as u32) * u32::from(self.proportion_cutoff_pct) / 100;

        max_count < threshold
    }

    /// Entropy degradation check.
    ///
    /// A healthy entropy source should produce diverse byte values.
    /// If fewer than 25% of possible byte values appear in a sample of
    /// sufficient length, entropy quality is degraded.
    fn check_degradation(&self, sample: &[u8]) -> bool {
        if sample.len() < 64 {
            return true; // too short to evaluate
        }

        let mut seen = [false; 256];
        for &byte in sample {
            seen[byte as usize] = true;
        }
        let unique = seen.iter().filter(|&&s| s).count();

        let min_unique = (sample.len() / 4).min(256);
        unique >= min_unique
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HealthMonitor {
    fn drop(&mut self) {
        // Z4 fix: zeroize last observed entropy data.
        self.proportion_window.zeroize();
        self.last_byte = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_on_diverse_input() {
        let mut monitor = HealthMonitor::new();
        let sample: Vec<u8> = (0..=255).collect();
        monitor.observe(&sample);
        assert_eq!(monitor.status(), HealthStatus::Healthy);
    }

    #[test]
    fn fails_on_per_byte_repetition() {
        // H3 fix: per-byte repetition, not per-sample.
        let mut monitor = HealthMonitor::new();
        // 20+ identical consecutive bytes should trigger failure.
        let sample = vec![0xAA; 64];
        monitor.observe(&sample);
        assert_eq!(monitor.status(), HealthStatus::Failed);
    }

    #[test]
    fn warns_on_low_diversity() {
        let mut monitor = HealthMonitor::new();
        let sample: Vec<u8> = (0..128).map(|i| if i % 2 == 0 { 0 } else { 1 }).collect();
        monitor.observe(&sample);
        assert!(monitor.status() != HealthStatus::Healthy);
    }

    #[test]
    fn recovery_requires_sustained_healthy_observations() {
        // H2 fix: single good observation should not clear Failed.
        let mut monitor = HealthMonitor::new();

        // Trigger failure with per-byte repetition.
        let bad = vec![0xBB; 64];
        monitor.observe(&bad);
        assert_eq!(monitor.status(), HealthStatus::Failed);

        // One good observation — should still be Failed.
        let good: Vec<u8> = (0..=255).collect();
        monitor.observe(&good);
        assert_eq!(
            monitor.status(),
            HealthStatus::Failed,
            "must not recover after single good observation"
        );

        // Feed enough consecutive good observations to recover.
        for _ in 0..RECOVERY_THRESHOLD {
            monitor.observe(&good);
        }
        assert_eq!(monitor.status(), HealthStatus::Healthy);
    }

    #[test]
    fn empty_sample_does_not_panic() {
        let mut monitor = HealthMonitor::new();
        monitor.observe(&[]);
        assert_eq!(monitor.status(), HealthStatus::Healthy);
    }

    #[test]
    fn single_byte_sample_does_not_panic() {
        let mut monitor = HealthMonitor::new();
        monitor.observe(&[0x42]);
        // Should stay Healthy — too short to trigger degradation.
        assert_eq!(monitor.status(), HealthStatus::Healthy);
    }

    #[test]
    fn large_sample_does_not_oom() {
        let mut monitor = HealthMonitor::new();
        // MAX_SAMPLE_SIZE = 65536; send something larger.
        let big = vec![0xAB; 100_000];
        monitor.observe(&big);
        // Should not panic or OOM. Will fail on repetition count test.
        assert_eq!(monitor.status(), HealthStatus::Failed);
    }
}
