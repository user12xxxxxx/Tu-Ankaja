use serde::Serialize;
use std::fmt;

use crate::health::HealthStatus;

// ---------------------------------------------------------------------------
// Trust levels — classify where entropy comes from
// ---------------------------------------------------------------------------

/// How much we trust an entropy source.
///
/// This is the core trust boundary: hardware noise is the only source that
/// should back key material. Timing jitter is supplemental. Simulated
/// sources are dev-only and must NEVER back production cryptographic output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrustLevel {
    /// Physical noise source: MOSFET electronic noise via MYOSA ADC.
    /// This is the only source trusted for cryptographic key material.
    Hardware,
    /// Timing jitter: interrupt timing, scheduling noise.
    /// Supplemental entropy — mixed in but not solely relied upon.
    Timing,
    /// Simulated xorshift64 seeded from system clock.
    /// NEVER trusted for production output. Dev/testing only.
    Simulated,
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hardware => write!(f, "hardware"),
            Self::Timing => write!(f, "timing"),
            Self::Simulated => write!(f, "simulated"),
        }
    }
}

// ---------------------------------------------------------------------------
// Output classification — different outputs need different entropy levels
// ---------------------------------------------------------------------------

/// Classification of cryptographic output by security requirement.
///
/// An AES-256 key protecting disk encryption requires stronger entropy
/// guarantees than a random hex string for a debug trace ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    /// AES keys, HMAC keys, master secrets. Requires full entropy.
    CryptoKey,
    /// Session tokens, API keys. Requires strong entropy.
    SessionToken,
    /// Passwords. Requires adequate entropy.
    Password,
    /// Raw random bytes, hex strings. Minimum requirements.
    Raw,
}

impl OutputClass {
    /// Human-readable label for error messages.
    pub fn label(&self) -> &'static str {
        match self {
            Self::CryptoKey => "cryptographic key",
            Self::SessionToken => "session token",
            Self::Password => "password",
            Self::Raw => "random bytes",
        }
    }
}

// ---------------------------------------------------------------------------
// Security policy — configurable rules that gate output generation
// ---------------------------------------------------------------------------

/// Security policy defining the minimum conditions for generating output.
///
/// This is the central enforcement point. Every output generation call
/// passes through the policy before any bytes leave the DRBG.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Minimum weighted pool entropy (bits) for each output class.
    pub min_entropy_crypto_key: f64,
    pub min_entropy_session_token: f64,
    pub min_entropy_password: f64,
    pub min_entropy_raw: f64,

    /// Whether to allow ANY cryptographic output from simulated sources.
    /// Default: true in dev, should be false in production.
    pub allow_simulated_crypto: bool,

    /// Whether to allow output when health status is Warning (not Failed).
    /// Failed always blocks output regardless of this setting.
    pub allow_warning_output: bool,

    /// Maximum total bytes that can be generated from a simulated source
    /// before the system refuses further output. Prevents accidentally
    /// running simulated mode in production for extended periods.
    pub simulated_output_limit: u64,

    /// Whether to require the pool to be well-seeded (≥256 bits weighted entropy)
    /// before allowing ANY output. Critical for production: prevents generating
    /// cryptographic material before sufficient entropy has been accumulated.
    pub require_pool_ready: bool,
}

impl SecurityPolicy {
    /// Strict production policy: requires hardware entropy, full pool seeding.
    pub fn production() -> Self {
        Self {
            min_entropy_crypto_key: 256.0,
            min_entropy_session_token: 192.0,
            min_entropy_password: 128.0,
            min_entropy_raw: 64.0,
            allow_simulated_crypto: false,
            allow_warning_output: false,
            simulated_output_limit: 0,
            require_pool_ready: true,
        }
    }

    /// Permissive development policy: allows simulated sources.
    pub fn development() -> Self {
        Self {
            min_entropy_crypto_key: 0.0,
            min_entropy_session_token: 0.0,
            min_entropy_password: 0.0,
            min_entropy_raw: 0.0,
            allow_simulated_crypto: true,
            allow_warning_output: true,
            simulated_output_limit: u64::MAX,
            require_pool_ready: false,
        }
    }

    /// Get the minimum entropy threshold for an output class.
    pub fn min_entropy_for(&self, class: OutputClass) -> f64 {
        match class {
            OutputClass::CryptoKey => self.min_entropy_crypto_key,
            OutputClass::SessionToken => self.min_entropy_session_token,
            OutputClass::Password => self.min_entropy_password,
            OutputClass::Raw => self.min_entropy_raw,
        }
    }
}

impl Default for SecurityPolicy {
    /// Default policy is development-permissive. Production deployments
    /// MUST explicitly set `SecurityPolicy::production()`.
    fn default() -> Self {
        Self::development()
    }
}

// ---------------------------------------------------------------------------
// Security events — auditable record of security-relevant state changes
// ---------------------------------------------------------------------------

/// Auditable security event. Every security-relevant action produces an event
/// that can be logged, forwarded to the frontend, or stored for forensics.
#[derive(Debug, Clone, Serialize)]
pub enum SecurityEvent {
    /// A new entropy source was registered.
    SourceRegistered {
        source_id: u8,
        trust_level: TrustLevel,
    },
    /// Health status transitioned.
    HealthStateChange {
        from: HealthStatus,
        to: HealthStatus,
    },
    /// Output was blocked by security policy.
    PolicyViolation {
        output_class: String,
        reason: String,
    },
    /// Cryptographic output was successfully generated.
    OutputGenerated {
        output_class: String,
        bytes: usize,
        pool_entropy_bits: f64,
    },
    /// Pool crossed the well-seeded threshold.
    PoolSeeded { entropy_bits: f64 },
    /// System is running in simulated mode (no hardware entropy).
    SimulatedModeActive,
    /// Simulated output limit was reached.
    SimulatedLimitReached { bytes_generated: u64, limit: u64 },
    /// Initial seeding completed.
    InitialSeedingComplete {
        trust_level: TrustLevel,
        health_ok: bool,
    },
}

// ---------------------------------------------------------------------------
// Security gate — enforces policy, records events
// ---------------------------------------------------------------------------

/// The security gate sits between the entropy pipeline and output generation.
/// It enforces the security policy, tracks trust context, and records events.
pub struct SecurityGate {
    policy: SecurityPolicy,
    trust_level: TrustLevel,
    events: Vec<SecurityEvent>,
    last_health: HealthStatus,
    simulated_bytes_generated: u64,
}

impl SecurityGate {
    pub fn new(policy: SecurityPolicy, trust_level: TrustLevel) -> Self {
        let mut gate = Self {
            policy,
            trust_level,
            events: Vec::new(),
            last_health: HealthStatus::Healthy,
            simulated_bytes_generated: 0,
        };

        gate.events.push(SecurityEvent::SourceRegistered {
            source_id: 0,
            trust_level,
        });

        if trust_level == TrustLevel::Simulated {
            gate.events.push(SecurityEvent::SimulatedModeActive);
            log::warn!("entropy engine running in SIMULATED mode — not suitable for production");
        }

        gate
    }

    /// Check whether generating output of the given class is allowed.
    ///
    /// Returns `Ok(())` if the policy permits, or an error describing why not.
    pub fn check_output_allowed(
        &mut self,
        class: OutputClass,
        bytes: usize,
        health: HealthStatus,
        pool_entropy_bits: f64,
        pool_well_seeded: bool,
    ) -> Result<(), SecurityViolation> {
        // Track health transitions.
        if health != self.last_health {
            self.events.push(SecurityEvent::HealthStateChange {
                from: self.last_health,
                to: health,
            });
            self.last_health = health;
        }

        // Gate 0: startup entropy safety — refuse ALL output until pool is ready.
        if self.policy.require_pool_ready && !pool_well_seeded {
            let reason = format!(
                "pool not yet well-seeded ({:.1} bits) — refusing {} generation",
                pool_entropy_bits,
                class.label()
            );
            self.events.push(SecurityEvent::PolicyViolation {
                output_class: class.label().to_string(),
                reason,
            });
            return Err(SecurityViolation::PoolNotReady { pool_entropy_bits });
        }

        // Gate 1: health status.
        if health == HealthStatus::Failed {
            let reason = "entropy source health check failed".to_string();
            self.events.push(SecurityEvent::PolicyViolation {
                output_class: class.label().to_string(),
                reason: reason.clone(),
            });
            return Err(SecurityViolation::HealthFailed);
        }

        if health == HealthStatus::Warning && !self.policy.allow_warning_output {
            let reason = "entropy source health warning (strict policy)".to_string();
            self.events.push(SecurityEvent::PolicyViolation {
                output_class: class.label().to_string(),
                reason: reason.clone(),
            });
            return Err(SecurityViolation::HealthWarning);
        }

        // Gate 2: simulated source restrictions.
        if self.trust_level == TrustLevel::Simulated {
            if !self.policy.allow_simulated_crypto
                && matches!(class, OutputClass::CryptoKey | OutputClass::SessionToken)
            {
                let reason = format!(
                    "simulated source cannot produce {} (policy: allow_simulated_crypto=false)",
                    class.label()
                );
                self.events.push(SecurityEvent::PolicyViolation {
                    output_class: class.label().to_string(),
                    reason: reason.clone(),
                });
                return Err(SecurityViolation::SimulatedSourceBlocked {
                    output_class: class,
                });
            }

            if self.simulated_bytes_generated + bytes as u64 > self.policy.simulated_output_limit {
                self.events.push(SecurityEvent::SimulatedLimitReached {
                    bytes_generated: self.simulated_bytes_generated,
                    limit: self.policy.simulated_output_limit,
                });
                return Err(SecurityViolation::SimulatedLimitExceeded {
                    generated: self.simulated_bytes_generated,
                    limit: self.policy.simulated_output_limit,
                });
            }
        }

        // Gate 3: minimum entropy threshold.
        let required = self.policy.min_entropy_for(class);
        if pool_entropy_bits < required {
            let reason = format!(
                "insufficient pool entropy for {}: have {:.1} bits, need {:.1} bits",
                class.label(),
                pool_entropy_bits,
                required,
            );
            self.events.push(SecurityEvent::PolicyViolation {
                output_class: class.label().to_string(),
                reason: reason.clone(),
            });
            return Err(SecurityViolation::InsufficientEntropy {
                output_class: class,
                have_bits: pool_entropy_bits,
                need_bits: required,
            });
        }

        // All gates passed — record successful generation.
        if self.trust_level == TrustLevel::Simulated {
            self.simulated_bytes_generated += bytes as u64;
        }

        self.events.push(SecurityEvent::OutputGenerated {
            output_class: class.label().to_string(),
            bytes,
            pool_entropy_bits,
        });

        Ok(())
    }

    /// Record that initial seeding completed.
    pub fn record_initial_seeding(&mut self, health_ok: bool) {
        self.events.push(SecurityEvent::InitialSeedingComplete {
            trust_level: self.trust_level,
            health_ok,
        });
        if !health_ok {
            log::warn!("initial seeding completed with health check failure");
        }
    }

    /// Record that the pool crossed the well-seeded threshold.
    pub fn record_pool_seeded(&mut self, entropy_bits: f64) {
        self.events.push(SecurityEvent::PoolSeeded { entropy_bits });
    }

    pub fn trust_level(&self) -> TrustLevel {
        self.trust_level
    }

    pub fn policy(&self) -> &SecurityPolicy {
        &self.policy
    }

    /// Drain all recorded security events (for forwarding to log/frontend).
    pub fn drain_events(&mut self) -> Vec<SecurityEvent> {
        std::mem::take(&mut self.events)
    }

    /// Read events without draining.
    pub fn events(&self) -> &[SecurityEvent] {
        &self.events
    }

    pub fn is_simulated(&self) -> bool {
        self.trust_level == TrustLevel::Simulated
    }
}

// ---------------------------------------------------------------------------
// Security violation — returned when the policy blocks output
// ---------------------------------------------------------------------------

/// Specific reason why the security gate blocked output generation.
#[derive(Debug, Clone)]
pub enum SecurityViolation {
    PoolNotReady {
        pool_entropy_bits: f64,
    },
    HealthFailed,
    HealthWarning,
    SimulatedSourceBlocked {
        output_class: OutputClass,
    },
    SimulatedLimitExceeded {
        generated: u64,
        limit: u64,
    },
    InsufficientEntropy {
        output_class: OutputClass,
        have_bits: f64,
        need_bits: f64,
    },
}

impl fmt::Display for SecurityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolNotReady { pool_entropy_bits } => {
                write!(
                    f,
                    "pool not yet well-seeded ({pool_entropy_bits:.1} bits) — refusing output until sufficient entropy accumulated"
                )
            }
            Self::HealthFailed => write!(f, "entropy source health check failed"),
            Self::HealthWarning => {
                write!(f, "entropy source health warning (strict policy)")
            }
            Self::SimulatedSourceBlocked { output_class } => {
                write!(
                    f,
                    "simulated source cannot produce {}",
                    output_class.label()
                )
            }
            Self::SimulatedLimitExceeded { generated, limit } => {
                write!(
                    f,
                    "simulated output limit exceeded: {generated} bytes generated, limit {limit}"
                )
            }
            Self::InsufficientEntropy {
                output_class,
                have_bits,
                need_bits,
            } => {
                write!(
                    f,
                    "insufficient entropy for {}: have {have_bits:.1} bits, need {need_bits:.1}",
                    output_class.label()
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_policy_allows_simulated_output() {
        let mut gate = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Simulated);
        let result = gate.check_output_allowed(
            OutputClass::CryptoKey,
            32,
            HealthStatus::Healthy,
            0.0, // no entropy
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn production_policy_blocks_simulated_keys() {
        let mut gate = SecurityGate::new(SecurityPolicy::production(), TrustLevel::Simulated);
        let result = gate.check_output_allowed(
            OutputClass::CryptoKey,
            32,
            HealthStatus::Healthy,
            1000.0,
            true,
        );
        assert!(matches!(
            result,
            Err(SecurityViolation::SimulatedSourceBlocked { .. })
        ));
    }

    #[test]
    fn production_policy_blocks_insufficient_entropy() {
        let mut gate = SecurityGate::new(SecurityPolicy::production(), TrustLevel::Hardware);
        let result = gate.check_output_allowed(
            OutputClass::CryptoKey,
            32,
            HealthStatus::Healthy,
            100.0, // need 256
            true,
        );
        assert!(matches!(
            result,
            Err(SecurityViolation::InsufficientEntropy { .. })
        ));
    }

    #[test]
    fn production_policy_allows_sufficient_entropy() {
        let mut gate = SecurityGate::new(SecurityPolicy::production(), TrustLevel::Hardware);
        let result = gate.check_output_allowed(
            OutputClass::CryptoKey,
            32,
            HealthStatus::Healthy,
            300.0,
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn health_failed_always_blocks() {
        let mut gate = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Hardware);
        let result =
            gate.check_output_allowed(OutputClass::Raw, 32, HealthStatus::Failed, 1000.0, true);
        assert!(matches!(result, Err(SecurityViolation::HealthFailed)));
    }

    #[test]
    fn production_blocks_warning_strict_blocks_dev_allows() {
        // Production: Warning blocked.
        let mut prod = SecurityGate::new(SecurityPolicy::production(), TrustLevel::Hardware);
        let result =
            prod.check_output_allowed(OutputClass::Raw, 32, HealthStatus::Warning, 1000.0, true);
        assert!(matches!(result, Err(SecurityViolation::HealthWarning)));

        // Dev: Warning allowed.
        let mut dev = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Hardware);
        let result =
            dev.check_output_allowed(OutputClass::Raw, 32, HealthStatus::Warning, 0.0, true);
        assert!(result.is_ok());
    }

    #[test]
    fn simulated_output_limit_enforced() {
        let mut policy = SecurityPolicy::development();
        policy.simulated_output_limit = 100;
        let mut gate = SecurityGate::new(policy, TrustLevel::Simulated);

        // First call: 64 bytes OK.
        assert!(gate
            .check_output_allowed(OutputClass::Raw, 64, HealthStatus::Healthy, 0.0, true)
            .is_ok());

        // Second call: 64 more → over limit.
        let result =
            gate.check_output_allowed(OutputClass::Raw, 64, HealthStatus::Healthy, 0.0, true);
        assert!(matches!(
            result,
            Err(SecurityViolation::SimulatedLimitExceeded { .. })
        ));
    }

    #[test]
    fn events_are_recorded() {
        let mut gate = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Simulated);
        gate.check_output_allowed(OutputClass::Raw, 16, HealthStatus::Healthy, 0.0, true)
            .unwrap();
        let events = gate.drain_events();
        // Should have: SourceRegistered, SimulatedModeActive, OutputGenerated
        assert!(events.len() >= 3);
    }

    #[test]
    fn health_transition_recorded() {
        let mut gate = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Hardware);
        // Start with Healthy, then Warning.
        gate.check_output_allowed(OutputClass::Raw, 16, HealthStatus::Healthy, 0.0, true)
            .unwrap();
        gate.check_output_allowed(OutputClass::Raw, 16, HealthStatus::Warning, 0.0, true)
            .unwrap();
        let events = gate.drain_events();
        let transitions: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SecurityEvent::HealthStateChange { .. }))
            .collect();
        assert_eq!(transitions.len(), 1);
    }

    #[test]
    fn production_blocks_when_pool_not_ready() {
        let mut gate = SecurityGate::new(SecurityPolicy::production(), TrustLevel::Hardware);
        // Pool NOT well-seeded: should block even Raw output.
        let result = gate.check_output_allowed(
            OutputClass::Raw,
            32,
            HealthStatus::Healthy,
            100.0,
            false, // pool not ready
        );
        assert!(
            matches!(result, Err(SecurityViolation::PoolNotReady { .. })),
            "production policy must block output when pool not ready"
        );
    }

    #[test]
    fn dev_allows_when_pool_not_ready() {
        let mut gate = SecurityGate::new(SecurityPolicy::development(), TrustLevel::Simulated);
        // Dev policy: pool not ready is OK.
        let result = gate.check_output_allowed(
            OutputClass::CryptoKey,
            32,
            HealthStatus::Healthy,
            0.0,
            false,
        );
        assert!(
            result.is_ok(),
            "dev policy should allow output even when pool not ready"
        );
    }

    #[test]
    fn lower_output_class_has_lower_threshold() {
        let policy = SecurityPolicy::production();
        assert!(
            policy.min_entropy_for(OutputClass::CryptoKey)
                > policy.min_entropy_for(OutputClass::Password)
        );
        assert!(
            policy.min_entropy_for(OutputClass::Password)
                > policy.min_entropy_for(OutputClass::Raw)
        );
    }
}
