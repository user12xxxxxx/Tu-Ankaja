use crate::health::HealthStatus;
use crate::security::SecurityViolation;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntropyError {
    #[error("serial port open failed on '{port}': {reason}")]
    SerialOpen { port: String, reason: String },

    #[error("serial read failed: {reason}")]
    SerialRead { reason: String },

    #[error("serial error: {reason}")]
    SerialError { reason: String },

    #[error("packet parse error: {reason}")]
    ParseError { reason: String },

    #[error("entropy health check failed: {status}")]
    HealthFailure { status: HealthStatus },

    #[error("entropy pool exhausted, reseed required")]
    PoolExhausted,

    #[error("DRBG reseed required after {bytes_generated} bytes")]
    DrbgReseedRequired { bytes_generated: u64 },

    #[error("invalid length: requested {requested}, max {max}")]
    InvalidLength { requested: usize, max: usize },

    #[error("security policy violation: {violation}")]
    SecurityViolation { violation: SecurityViolation },

    #[error("MQTT error: {reason}")]
    MqttError { reason: String },
}
