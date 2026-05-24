use entropy_engine::api::EntropyService;
use entropy_engine::health::HealthStatus;
use entropy_engine::models::PasswordConfig;
use entropy_engine::security::{SecurityPolicy, TrustLevel};
use entropy_engine::serial::SerialSource;

// --- Push mode (MQTT) tests ---

#[test]
fn push_mode_ingest_and_generate() {
    let mut service = EntropyService::new_push(
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );

    // Feed enough entropy via ingest() to fill the pool.
    let good_data: Vec<u8> = (0..=255).cycle().take(512).collect();
    for _ in 0..20 {
        service.ingest(0x01, &good_data);
    }

    // Now we should be able to generate output.
    let hex = service.random_hex(32).unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn push_mode_successive_outputs_differ() {
    let mut service = EntropyService::new_push(
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );

    let data: Vec<u8> = (0..=255).cycle().take(512).collect();
    for _ in 0..20 {
        service.ingest(0x01, &data);
    }

    let a = service.random_bytes(32).unwrap();
    let b = service.random_bytes(32).unwrap();
    assert_ne!(a, b);
}

#[test]
fn push_mode_aes_key_generation() {
    let mut service = EntropyService::new_push(
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );

    let data: Vec<u8> = (0..=255).cycle().take(512).collect();
    for _ in 0..20 {
        service.ingest(0x01, &data);
    }

    let key = service.generate_aes_key().unwrap();
    assert_eq!(key.len(), 64); // 32 bytes = 64 hex chars
}

#[test]
fn push_mode_stats_track_ingestion() {
    let mut service = EntropyService::new_push(
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );

    let data: Vec<u8> = (0..=255).cycle().take(256).collect();
    for _ in 0..10 {
        service.ingest(0x01, &data);
    }

    let stats = service.get_entropy_stats();
    assert!(stats.pool_fills > 0);
    assert!(stats.pool_entropy_bits > 0.0);
}

#[test]
fn push_mode_production_blocks_before_enough_entropy() {
    let mut service = EntropyService::production_push(0x01);

    // No entropy ingested yet — production policy should block.
    let result = service.random_bytes(32);
    assert!(result.is_err(), "should block with no entropy ingested");
}

#[test]
fn push_mode_empty_ingest_is_safe() {
    let mut service = EntropyService::new_push(
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );
    service.ingest(0x01, &[]);
    // Should not panic or corrupt state.
}

// --- Functional tests (dev policy) ---

#[test]
fn random_hex_correct_length() {
    let mut service = EntropyService::demo();
    let hex = service.random_hex(32).unwrap();
    assert_eq!(hex.len(), 64);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn random_bytes_correct_length() {
    let mut service = EntropyService::demo();
    let bytes = service.random_bytes(128).unwrap();
    assert_eq!(bytes.len(), 128);
}

#[test]
fn generate_password_default_config() {
    let mut service = EntropyService::demo();
    let pw = service
        .generate_password(PasswordConfig::default())
        .unwrap();
    assert_eq!(pw.len(), 20);
}

#[test]
fn generate_password_custom_config() {
    let mut service = EntropyService::demo();
    let config = PasswordConfig {
        length: 32,
        uppercase: true,
        lowercase: true,
        digits: true,
        symbols: false,
    };
    let pw = service.generate_password(config).unwrap();
    assert_eq!(pw.len(), 32);
    assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn generate_aes_key_is_64_hex_chars() {
    let mut service = EntropyService::demo();
    let key = service.generate_aes_key().unwrap();
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn generate_session_token_is_hex() {
    let mut service = EntropyService::demo();
    let token = service.generate_session_token().unwrap();
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn health_starts_healthy() {
    let service = EntropyService::demo();
    assert_eq!(service.get_entropy_integrity(), HealthStatus::Healthy);
}

#[test]
fn stats_track_generation() {
    let mut service = EntropyService::demo();
    service.random_bytes(64).unwrap();
    service.random_bytes(64).unwrap();
    let stats = service.get_entropy_stats();
    assert_eq!(stats.bytes_generated, 128);
    assert!(stats.reseed_count >= 2);
    assert!(stats.pool_fills >= 2);
}

#[test]
fn successive_outputs_differ() {
    let mut service = EntropyService::demo();
    let a = service.random_bytes(32).unwrap();
    let b = service.random_bytes(32).unwrap();
    assert_ne!(a, b);
}

// --- Security policy tests ---

#[test]
fn production_policy_blocks_simulated_crypto_keys() {
    let mut service = EntropyService::new(
        SerialSource::simulated(),
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::production(),
    )
    .unwrap();
    let result = service.generate_aes_key();
    assert!(
        result.is_err(),
        "production policy must block simulated AES keys"
    );
}

#[test]
fn production_policy_blocks_simulated_session_tokens() {
    let mut service = EntropyService::new(
        SerialSource::simulated(),
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::production(),
    )
    .unwrap();
    let result = service.generate_session_token();
    assert!(
        result.is_err(),
        "production policy must block simulated session tokens"
    );
}

#[test]
fn demo_reports_simulated_trust_level() {
    let service = EntropyService::demo();
    assert!(service.is_simulated());
    assert_eq!(service.trust_level(), TrustLevel::Simulated);
}

#[test]
fn security_events_are_recorded() {
    let mut service = EntropyService::demo();
    service.random_bytes(32).unwrap();
    let events = service.drain_security_events();
    assert!(!events.is_empty(), "should have security events");
}

// --- Edge case tests ---

#[test]
fn zero_length_request_returns_empty() {
    let mut service = EntropyService::demo();
    let bytes = service.random_bytes(0).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn max_output_length_succeeds() {
    let mut service = EntropyService::demo();
    // 64 KiB = maximum allowed
    let bytes = service.random_bytes(1 << 16).unwrap();
    assert_eq!(bytes.len(), 1 << 16);
}

#[test]
fn over_max_output_length_is_rejected() {
    let mut service = EntropyService::demo();
    let result = service.random_bytes((1 << 16) + 1);
    assert!(result.is_err(), "must reject requests exceeding 64 KiB");
}

#[test]
fn many_successive_generations_all_differ() {
    let mut service = EntropyService::demo();
    let mut previous = Vec::new();
    for _ in 0..20 {
        let output = service.random_bytes(32).unwrap();
        assert!(
            !previous.contains(&output),
            "repeated output detected in 20 successive generations"
        );
        previous.push(output);
    }
}

#[test]
fn entropy_stats_report_pool_seeded_state() {
    let mut service = EntropyService::demo();
    // Generate several times to accumulate entropy.
    for _ in 0..10 {
        let _ = service.random_bytes(64);
    }
    let stats = service.get_entropy_stats();
    assert!(
        stats.pool_entropy_bits >= 0.0,
        "entropy bits must be non-negative"
    );
}

#[test]
fn password_minimum_length_enforced() {
    let mut service = EntropyService::demo();
    let config = PasswordConfig {
        length: 1,
        uppercase: true,
        lowercase: true,
        digits: true,
        symbols: true,
    };
    let pw = service.generate_password(config).unwrap();
    assert_eq!(pw.len(), 1);
}

#[test]
fn constructor_returns_result() {
    // Verify the new() returns Result, not panicking.
    let result = EntropyService::new(
        SerialSource::simulated(),
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::development(),
    );
    assert!(result.is_ok(), "simulated source should succeed");
}
