use std::sync::{Arc, Mutex};

use entropy_engine::api::EntropyService;
use entropy_engine::blockchain::BlockchainValidator;
use entropy_engine::models::PasswordConfig;
use entropy_engine::mqtt::{MqttConfig, MqttIngestor};
use entropy_engine::otp::OtpService;
use entropy_engine::otp_mqtt::{OtpMqttConfig, OtpMqttIngestor};
use entropy_engine::security::{SecurityPolicy, TrustLevel};
use entropy_engine::server::UnifiedState;
use entropy_engine::serial::SerialSource;

fn main() {
    env_logger::init();

    // Check if MQTT mode is requested via environment variable.
    // Usage: ENTROPY_MODE=mqtt cargo run
    //        ENTROPY_MODE=mqtt MQTT_HOST=192.168.1.50 cargo run
    let mode = std::env::var("ENTROPY_MODE").unwrap_or_default();

    if mode == "otp" {
        run_otp_mode();
    } else if mode == "mqtt" {
        run_mqtt_mode();
    } else {
        run_serial_mode();
    }
}

/// OTP mode — subscribes to random/numbers and random/params MQTT topics,
/// serves OTP generation AND entropy engine via unified HTTP API.
fn run_otp_mode() {
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".into());
    let mqtt_port: u16 = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);

    println!("=== Entropy Vault Engine (Unified Mode) ===");
    println!("broker:  {}:{}", mqtt_host, mqtt_port);
    println!("topics:  random/numbers, random/params, random/MAC");

    // Create both services
    let otp_service = Arc::new(Mutex::new(OtpService::new()));
    let entropy_service = Arc::new(Mutex::new(EntropyService::new_push(
        0x01,
        TrustLevel::Hardware,
        SecurityPolicy::development(),
    )));

    let mqtt_user = std::env::var("MQTT_USER").ok();
    let mqtt_pass = std::env::var("MQTT_PASS").ok();

    if mqtt_user.is_some() {
        println!("MQTT auth: user '{}'", mqtt_user.as_deref().unwrap_or(""));
    }

    let config = OtpMqttConfig {
        broker_host: mqtt_host,
        broker_port: mqtt_port,
        numbers_topic: "random/numbers".into(),
        params_topic: "random/params".into(),
        mac_topic: "random/MAC".into(),
        client_id: format!("entropy-vault-otp-{}", std::process::id()),
        username: mqtt_user,
        password: mqtt_pass,
    };

    let blockchain = Arc::new(BlockchainValidator::from_env());
    println!("blockchain: MultiChain validator initialized");

    let _ingestor = match OtpMqttIngestor::start(
        config,
        Arc::clone(&otp_service),
        blockchain,
        Some(Arc::clone(&entropy_service)),
    ) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to start OTP MQTT ingestor: {e}");
            eprintln!("Is Mosquitto running? Try: mosquitto -d");
            return;
        }
    };

    let api_port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let state = UnifiedState {
        otp: otp_service,
        entropy: entropy_service,
    };

    println!("Starting unified HTTP API server...");
    println!("Open http://localhost:3000 (Next.js) to see all pages.\n");

    if let Err(e) = entropy_engine::server::run_unified(state, api_port) {
        eprintln!("HTTP server error: {e}");
    }
}

/// Original serial/simulated mode (pull model).
fn run_serial_mode() {
    let mut service = EntropyService::demo();

    println!("=== Entropy Vault Engine (Serial Mode) ===");
    println!(
        "mode:   {} (trust: {})",
        if service.is_simulated() { "simulated" } else { "hardware" },
        service.trust_level()
    );
    println!("health: {}", service.health_report());

    generate_and_print(&mut service);
    print_stats(&mut service);
    print_production_policy_test();
}

/// MQTT push mode — connects to Mosquitto broker, receives entropy wirelessly.
fn run_mqtt_mode() {
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".into());
    let mqtt_port: u16 = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);
    let mqtt_topic = std::env::var("MQTT_TOPIC")
        .unwrap_or_else(|_| "entropy-vault/raw".into());

    println!("=== Entropy Vault Engine (MQTT Mode) ===");
    println!("broker: {}:{}", mqtt_host, mqtt_port);
    println!("topic:  {}", mqtt_topic);

    // Create a push-mode service with development policy for demo.
    // In production, use EntropyService::production_push(0x01) which
    // will block output until enough real entropy has accumulated.
    let service = Arc::new(Mutex::new(EntropyService::new_push(
        0x01,
        TrustLevel::Hardware,
        SecurityPolicy::development(),
    )));

    let mqtt_user = std::env::var("MQTT_USER").ok();
    let mqtt_pass = std::env::var("MQTT_PASS").ok();

    if mqtt_user.is_some() {
        println!("MQTT auth: user '{}'", mqtt_user.as_deref().unwrap_or(""));
    }

    let config = MqttConfig {
        broker_host: mqtt_host,
        broker_port: mqtt_port,
        topic: mqtt_topic,
        source_id: 0x01,
        client_id: "entropy-vault-engine".into(),
        username: mqtt_user,
        password: mqtt_pass,
    };

    // Start the MQTT ingestor — it feeds entropy into the service in the background.
    let _ingestor = match MqttIngestor::start(config, Arc::clone(&service)) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to start MQTT ingestor: {e}");
            eprintln!("Is Mosquitto running? Try: mosquitto -d");
            return;
        }
    };

    // Start the HTTP API server so the Next.js frontend can connect.
    // This blocks forever — the engine runs as a long-lived server.
    let api_port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    println!("Starting HTTP API server...");
    println!("Open http://localhost:3000 (Next.js) to see the dashboard.\n");

    if let Err(e) = entropy_engine::server::run(service, api_port) {
        eprintln!("HTTP server error: {e}");
    }
}

fn generate_and_print(service: &mut EntropyService) {
    match service.random_hex(32) {
        Ok(hex) => println!("random hex (32B): {hex}"),
        Err(e) => eprintln!("random_hex: {e}"),
    }

    match service.generate_password(PasswordConfig::default()) {
        Ok(pw) => println!("password (20ch): {pw}"),
        Err(e) => eprintln!("password: {e}"),
    }

    match service.generate_aes_key() {
        Ok(key) => println!("AES-256 key:     {key}"),
        Err(e) => eprintln!("aes_key: {e}"),
    }

    match service.generate_session_token() {
        Ok(token) => println!("session token:   {token}"),
        Err(e) => eprintln!("session_token: {e}"),
    }
}

fn print_stats(service: &mut EntropyService) {
    print_stats_from(service);

    // Show security events.
    let events = service.drain_security_events();
    println!("\n--- Security Events ({}) ---", events.len());
    for event in &events {
        println!("  {:?}", event);
    }
}

fn print_stats_from(service: &mut EntropyService) {
    let stats = service.get_entropy_stats();
    println!("\n--- Pipeline Stats ---");
    println!("pool fills:      {}", stats.pool_fills);
    println!("bytes generated: {}", stats.bytes_generated);
    println!(
        "pool entropy:    {:.1} bits (well-seeded: {})",
        stats.pool_entropy_bits, stats.pool_well_seeded
    );
    for sq in &stats.source_quality {
        println!(
            "  source 0x{:02X}: tier={:<10} H_min={:.2} b/B  confidence={:.2}",
            sq.source_id, sq.tier, sq.min_entropy_bits_per_byte, sq.confidence
        );
    }
}

fn print_production_policy_test() {
    println!("\n--- Production Policy Test ---");
    let mut prod = EntropyService::new(
        SerialSource::simulated(),
        0x01,
        TrustLevel::Simulated,
        SecurityPolicy::production(),
    )
    .expect("production service creation failed");

    match prod.generate_aes_key() {
        Ok(_) => println!("  AES key: generated (unexpected)"),
        Err(e) => println!("  AES key: BLOCKED — {e}"),
    }

    match prod.random_hex(16) {
        Ok(_) => println!("  raw hex: generated (unexpected under production policy)"),
        Err(e) => println!("  raw hex: BLOCKED — {e}"),
    }
}
