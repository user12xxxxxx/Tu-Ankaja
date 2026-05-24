use crate::crypto::CryptoOutput;
use crate::drbg::Drbg;
use crate::errors::EntropyError;
use crate::health::{HealthMonitor, HealthStatus};
use crate::models::{EntropyStats, PasswordConfig, SourceQualitySummary};
use crate::pool::EntropyPool;
use crate::security::{OutputClass, SecurityEvent, SecurityGate, SecurityPolicy, TrustLevel};
use crate::serial::SerialSource;
use crate::whitening::Whitener;

/// Maximum bytes that can be requested in a single API call.
const MAX_OUTPUT_LEN: usize = 1 << 16; // 64 KiB

/// Core entropy service that orchestrates the full pipeline:
///
/// **Pull mode** (serial): Serial → Health → Security Gate → Pool (SHA-256) → DRBG (ChaCha20) → Output
/// **Push mode** (MQTT):   ingest() feeds Health → Pool; pipeline does Gate → DRBG → Output
///
/// The security gate enforces the configured policy at every output point.
/// Different output classes (keys, tokens, passwords, raw) have different
/// entropy and trust requirements.
pub struct EntropyService {
    source: Option<SerialSource>,
    source_id: u8,
    pool: EntropyPool,
    drbg: Drbg,
    health: HealthMonitor,
    gate: SecurityGate,
    bytes_generated: u64,
    reseed_count: u64,
    pool_was_well_seeded: bool,
}

impl EntropyService {
    /// Create a pull-mode service with the given serial source, trust level, and policy.
    ///
    /// Pull mode: the pipeline reads entropy from the serial source on every call.
    /// Returns `Err` if the initial seeding read from the source fails.
    pub fn new(
        mut source: SerialSource,
        source_id: u8,
        trust_level: TrustLevel,
        policy: SecurityPolicy,
    ) -> Result<Self, EntropyError> {
        let mut gate = SecurityGate::new(policy, trust_level);

        // Initial seeding: collect, condition, seed pool + DRBG.
        let initial_raw = source.read_raw(256)?;
        let conditioned = Whitener::condition_fixed(&initial_raw);

        let mut pool = EntropyPool::from_seed(&conditioned);
        let drbg = Drbg::from_seed(pool.derive_seed());

        // Health-check the initial seed material.
        let mut health = HealthMonitor::new();
        health.observe(&initial_raw);
        let health_ok = health.status() != HealthStatus::Failed;
        gate.record_initial_seeding(health_ok);

        Ok(Self {
            source: Some(source),
            source_id,
            pool,
            drbg,
            health,
            gate,
            bytes_generated: 0,
            reseed_count: 0,
            pool_was_well_seeded: false,
        })
    }

    /// Create a push-mode service with no serial source.
    ///
    /// Push mode: entropy is fed externally via `ingest()` (e.g. from MQTT).
    /// The pool starts empty and accumulates entropy as `ingest()` is called.
    /// Output generation will be blocked by the security gate until enough
    /// entropy has been accumulated (if using production policy).
    pub fn new_push(
        source_id: u8,
        trust_level: TrustLevel,
        policy: SecurityPolicy,
    ) -> Self {
        let gate = SecurityGate::new(policy, trust_level);
        let pool = EntropyPool::new();
        let drbg = Drbg::from_seed([0u8; 32]);

        Self {
            source: None,
            source_id,
            pool,
            drbg,
            health: HealthMonitor::new(),
            gate,
            bytes_generated: 0,
            reseed_count: 0,
            pool_was_well_seeded: false,
        }
    }

    /// Create a development-mode service with simulated source and permissive policy.
    ///
    /// Panics only if the simulated source fails (should never happen).
    pub fn demo() -> Self {
        Self::new(
            SerialSource::simulated(),
            0x01,
            TrustLevel::Simulated,
            SecurityPolicy::development(),
        )
        .expect("simulated source should never fail initial seeding")
    }

    /// Create a production-mode service with the given hardware source (pull mode).
    pub fn production(source: SerialSource, source_id: u8) -> Result<Self, EntropyError> {
        Self::new(
            source,
            source_id,
            TrustLevel::Hardware,
            SecurityPolicy::production(),
        )
    }

    /// Create a production-mode push service for use with MQTT.
    pub fn production_push(source_id: u8) -> Self {
        Self::new_push(source_id, TrustLevel::Hardware, SecurityPolicy::production())
    }

    // -----------------------------------------------------------------------
    // Internal: the entropy pipeline
    // -----------------------------------------------------------------------

    /// Push entropy bytes into the pipeline from an external source (e.g. MQTT).
    ///
    /// This performs health checking and pool mixing — the same steps that
    /// `pipeline()` does in pull mode before the security gate. Call this
    /// from the MQTT ingestor thread whenever new entropy bytes arrive.
    pub fn ingest(&mut self, source_id: u8, raw: &[u8]) {
        if raw.is_empty() {
            return;
        }

        // Health check the incoming data.
        self.health.observe(raw);

        // Mix into the entropy pool with source quality tracking.
        self.pool.mix_from_source(source_id, raw);

        // Track pool seeding transition.
        if !self.pool_was_well_seeded && self.pool.is_well_seeded() {
            self.pool_was_well_seeded = true;
            self.gate
                .record_pool_seeded(self.pool.weighted_entropy_bits());
        }
    }

    /// Run the entropy pipeline and produce `length` random bytes,
    /// gated by security policy for the given output class.
    ///
    /// In pull mode (serial source): reads from source, then gates + generates.
    /// In push mode (MQTT): pool is already fed by `ingest()`, just gates + generates.
    fn pipeline(&mut self, length: usize, class: OutputClass) -> Result<Vec<u8>, EntropyError> {
        if length == 0 {
            return Ok(Vec::new());
        }
        if length > MAX_OUTPUT_LEN {
            return Err(EntropyError::InvalidLength {
                requested: length,
                max: MAX_OUTPUT_LEN,
            });
        }

        // In pull mode, collect entropy from the serial source.
        // In push mode, the pool is already fed by ingest() calls.
        if let Some(ref mut source) = self.source {
            let raw = source.read_raw(length.max(64))?;
            self.health.observe(&raw);
            self.pool.mix_from_source(self.source_id, &raw);

            if !self.pool_was_well_seeded && self.pool.is_well_seeded() {
                self.pool_was_well_seeded = true;
                self.gate
                    .record_pool_seeded(self.pool.weighted_entropy_bits());
            }
        }

        // SECURITY GATE: check policy before producing output.
        self.gate
            .check_output_allowed(
                class,
                length,
                self.health.status(),
                self.pool.weighted_entropy_bits(),
                self.pool.is_well_seeded(),
            )
            .map_err(|violation| EntropyError::SecurityViolation { violation })?;

        // Reseed DRBG from pool.
        self.drbg.reseed(self.pool.derive_seed());
        self.reseed_count += 1;

        // Generate output.
        let output = self.drbg.fill(length)?;
        self.bytes_generated += length as u64;

        Ok(output)
    }

    // -----------------------------------------------------------------------
    // Public API: each call declares its output class
    // -----------------------------------------------------------------------

    /// Generate random bytes (lowest security requirement).
    pub fn random_bytes(&mut self, length: usize) -> Result<Vec<u8>, EntropyError> {
        self.pipeline(length, OutputClass::Raw)
    }

    /// Generate a hex-encoded random string.
    pub fn random_hex(&mut self, length: usize) -> Result<String, EntropyError> {
        Ok(CryptoOutput::to_hex(&self.random_bytes(length)?))
    }

    /// Generate a cryptographically secure password.
    pub fn generate_password(&mut self, config: PasswordConfig) -> Result<String, EntropyError> {
        let charset = config.charset();
        if charset.is_empty() {
            return Err(EntropyError::InvalidLength {
                requested: config.length,
                max: 0,
            });
        }

        let charset_len = charset.len() as u16;
        let limit = (256 / charset_len) * charset_len;
        let expected_attempts = ((config.length as u64) * 256 / (limit as u64)) + 1;
        let raw_len = (expected_attempts as usize * 2).max(config.length * 3);
        let raw_len = raw_len.min(MAX_OUTPUT_LEN);

        let bytes = self.pipeline(raw_len, OutputClass::Password)?;
        CryptoOutput::generate_password(&bytes, &config)
    }

    /// Generate an AES-256 key (highest security requirement), returned as hex.
    pub fn generate_aes_key(&mut self) -> Result<String, EntropyError> {
        let bytes = self.pipeline(32, OutputClass::CryptoKey)?;
        let key = CryptoOutput::generate_aes256_key(&bytes)?;
        Ok(CryptoOutput::to_hex(&key))
    }

    /// Generate a session token (high security requirement), hex-encoded.
    pub fn generate_session_token(&mut self) -> Result<String, EntropyError> {
        let bytes = self.pipeline(32, OutputClass::SessionToken)?;
        Ok(CryptoOutput::generate_session_token(&bytes))
    }

    // -----------------------------------------------------------------------
    // Observability
    // -----------------------------------------------------------------------

    pub fn get_entropy_integrity(&self) -> HealthStatus {
        self.health.status()
    }

    pub fn get_entropy_stats(&self) -> EntropyStats {
        let source_quality: Vec<SourceQualitySummary> = self
            .pool
            .quality_tracker()
            .all_profiles()
            .iter()
            .map(|p| SourceQualitySummary::from(*p))
            .collect();

        EntropyStats {
            pool_fills: self.pool.fill_count(),
            bytes_generated: self.bytes_generated,
            reseed_count: self.reseed_count,
            health_status: self.health.status().to_string(),
            drbg_bytes_since_reseed: self.drbg.bytes_since_reseed(),
            pool_entropy_bits: self.pool.weighted_entropy_bits(),
            pool_well_seeded: self.pool.is_well_seeded(),
            source_quality,
        }
    }

    pub fn health_report(&self) -> HealthStatus {
        self.health.status()
    }

    /// Whether the engine is running against a simulated (non-hardware) source.
    pub fn is_simulated(&self) -> bool {
        self.gate.is_simulated()
    }

    /// Drain security events for logging or forwarding to the frontend.
    pub fn drain_security_events(&mut self) -> Vec<SecurityEvent> {
        self.gate.drain_events()
    }

    /// The active trust level.
    pub fn trust_level(&self) -> TrustLevel {
        self.gate.trust_level()
    }
}
