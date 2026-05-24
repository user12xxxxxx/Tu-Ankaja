use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::api::EntropyService;
use crate::errors::EntropyError;
use crate::models::PasswordConfig;
use crate::otp::OtpService;

type SharedOtpService = Arc<Mutex<OtpService>>;

type SharedService = Arc<Mutex<EntropyService>>;

/// Combined state for the unified server — holds both OTP and Entropy services.
#[derive(Clone)]
pub struct UnifiedState {
    pub otp: SharedOtpService,
    pub entropy: SharedService,
}

/// Start the HTTP API server on the given port.
///
/// This blocks the calling thread. The server provides REST endpoints
/// so the Next.js frontend can talk to the engine without Tauri.
pub fn run(service: SharedService, port: u16) -> Result<(), EntropyError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| EntropyError::MqttError {
        reason: format!("failed to create tokio runtime: {}", e),
    })?;

    rt.block_on(async move {
        let app = Router::new()
            .route("/api/integrity", get(get_integrity))
            .route("/api/stats", get(get_stats))
            .route("/api/events", get(get_events))
            .route("/api/generate/aes-key", post(generate_aes_key))
            .route("/api/generate/password", post(generate_password))
            .route("/api/generate/session-token", post(generate_session_token))
            .route("/api/generate/hex", post(generate_hex))
            .with_state(service)
            .layer(CorsLayer::permissive());

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            EntropyError::MqttError {
                reason: format!("failed to bind to {}: {}", addr, e),
            }
        })?;

        log::info!("HTTP API server listening on http://localhost:{}", port);
        println!("HTTP API: http://localhost:{}", port);

        axum::serve(listener, app)
            .await
            .map_err(|e| EntropyError::MqttError {
                reason: format!("HTTP server error: {}", e),
            })
    })
}

// ── Response types ──────────────────────────────────────────────────

#[derive(Serialize)]
struct IntegrityResponse {
    status: String,
    label: String,
    #[serde(rename = "checkedAt")]
    checked_at: String,
}

#[derive(Serialize)]
struct GeneratedResponse {
    value: String,
    timestamp: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct PasswordRequest {
    length: Option<usize>,
}

// ── Handlers ────────────────────────────────────────────────────────

async fn get_integrity(
    State(service): State<SharedService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    let status = svc.get_entropy_integrity();

    Json(IntegrityResponse {
        status: status.to_string(),
        label: format!("Integrity {}", status),
        checked_at: now_iso(),
    })
}

async fn get_stats(
    State(service): State<SharedService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    Json(svc.get_entropy_stats())
}

async fn get_events(
    State(service): State<SharedService>,
) -> impl IntoResponse {
    let mut svc = service.lock().unwrap();
    let events = svc.drain_security_events();

    let normalized: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            serde_json::json!({
                "kind": "event",
                "detail": format!("{:?}", e),
                "timestamp": now_iso(),
            })
        })
        .collect();

    Json(normalized)
}

async fn generate_aes_key(
    State(service): State<SharedService>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = service.lock().unwrap();
    match svc.generate_aes_key() {
        Ok(key) => Ok(Json(GeneratedResponse {
            value: key,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn generate_password(
    State(service): State<SharedService>,
    Json(body): Json<PasswordRequest>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let length = body.length.unwrap_or(24).clamp(12, 96);
    let config = PasswordConfig {
        length,
        ..PasswordConfig::default()
    };

    let mut svc = service.lock().unwrap();
    match svc.generate_password(config) {
        Ok(pw) => Ok(Json(GeneratedResponse {
            value: pw,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn generate_session_token(
    State(service): State<SharedService>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = service.lock().unwrap();
    match svc.generate_session_token() {
        Ok(token) => Ok(Json(GeneratedResponse {
            value: token,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn generate_hex(
    State(service): State<SharedService>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = service.lock().unwrap();
    match svc.random_hex(32) {
        Ok(hex) => Ok(Json(GeneratedResponse {
            value: hex,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn err_response(e: EntropyError) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: e.to_string(),
        }),
    )
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("1970-01-01T00:00:00Z+{}", now)
}

// ── OTP Server ─────────────────────────────────────────────────────

pub fn run_otp(service: SharedOtpService, port: u16) -> Result<(), EntropyError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| EntropyError::MqttError {
        reason: format!("failed to create tokio runtime: {}", e),
    })?;

    rt.block_on(async move {
        let app = Router::new()
            .route("/api/otp/generate", post(otp_generate))
            .route("/api/otp/history", get(otp_history))
            .route("/api/data/numbers", get(otp_numbers))
            .route("/api/data/params", get(otp_params))
            .route("/api/data/mac-status", get(otp_mac_status))
            .with_state(service)
            .layer(CorsLayer::permissive());

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            EntropyError::MqttError {
                reason: format!("failed to bind to {}: {}", addr, e),
            }
        })?;

        log::info!("OTP HTTP API server listening on http://localhost:{}", port);
        println!("OTP HTTP API: http://localhost:{}", port);

        axum::serve(listener, app)
            .await
            .map_err(|e| EntropyError::MqttError {
                reason: format!("HTTP server error: {}", e),
            })
    })
}

// ── OTP Response types ─────────────────────────────────────────────

#[derive(Serialize)]
struct OtpResponse {
    otp: String,
    source_number: u16,
    timestamp: String,
}

#[derive(Serialize)]
struct NumbersResponse {
    numbers: Vec<u16>,
    count: usize,
}

#[derive(Serialize)]
struct ParamsResponse {
    params: Vec<String>,
    count: usize,
}

#[derive(Serialize)]
struct OtpHistoryResponse {
    history: Vec<crate::otp::OtpRecord>,
}

// ── OTP Handlers ───────────────────────────────────────────────────

async fn otp_generate(
    State(service): State<SharedOtpService>,
) -> Result<Json<OtpResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = service.lock().unwrap();
    match svc.generate_otp() {
        Ok(record) => Ok(Json(OtpResponse {
            otp: record.otp,
            source_number: record.source_number,
            timestamp: record.created_at,
        })),
        Err(msg) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse { error: msg }),
        )),
    }
}

async fn otp_history(
    State(service): State<SharedOtpService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    let history: Vec<_> = svc.otp_history.iter().cloned().collect();
    Json(OtpHistoryResponse { history })
}

async fn otp_numbers(
    State(service): State<SharedOtpService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    let numbers: Vec<u16> = svc.random_numbers.iter().copied().collect();
    let count = numbers.len();
    Json(NumbersResponse { numbers, count })
}

async fn otp_params(
    State(service): State<SharedOtpService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    let params: Vec<String> = svc.sensor_params.iter().cloned().collect();
    let count = params.len();
    Json(ParamsResponse { params, count })
}

#[derive(Serialize)]
struct MacStatusResponse {
    mac: Option<String>,
    validated: Option<bool>,
    checked: bool,
}

async fn otp_mac_status(
    State(service): State<SharedOtpService>,
) -> impl IntoResponse {
    let svc = service.lock().unwrap();
    Json(MacStatusResponse {
        mac: svc.mac_address.clone(),
        validated: svc.mac_validated,
        checked: svc.mac_validated.is_some(),
    })
}

// ── Unified Server ──────────────────────────────────────────────────

/// Run a unified HTTP server that serves BOTH OTP and Entropy Engine routes.
/// This lets all three frontend pages (OTP, Data, Entropy) work simultaneously.
pub fn run_unified(state: UnifiedState, port: u16) -> Result<(), EntropyError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| EntropyError::MqttError {
        reason: format!("failed to create tokio runtime: {}", e),
    })?;

    rt.block_on(async move {
        let app = Router::new()
            // OTP routes
            .route("/api/otp/generate", post(u_otp_generate))
            .route("/api/otp/history", get(u_otp_history))
            .route("/api/data/numbers", get(u_otp_numbers))
            .route("/api/data/params", get(u_otp_params))
            .route("/api/data/mac-status", get(u_otp_mac_status))
            // Entropy Engine routes
            .route("/api/integrity", get(u_get_integrity))
            .route("/api/stats", get(u_get_stats))
            .route("/api/events", get(u_get_events))
            .route("/api/generate/aes-key", post(u_generate_aes_key))
            .route("/api/generate/password", post(u_generate_password))
            .route("/api/generate/session-token", post(u_generate_session_token))
            .route("/api/generate/hex", post(u_generate_hex))
            .with_state(state)
            .layer(CorsLayer::permissive());

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
            EntropyError::MqttError {
                reason: format!("failed to bind to {}: {}", addr, e),
            }
        })?;

        log::info!("Unified HTTP API server listening on http://localhost:{}", port);
        println!("Unified HTTP API: http://localhost:{}", port);
        println!("  OTP routes:     /api/otp/*, /api/data/*");
        println!("  Engine routes:  /api/integrity, /api/stats, /api/events, /api/generate/*");

        axum::serve(listener, app)
            .await
            .map_err(|e| EntropyError::MqttError {
                reason: format!("HTTP server error: {}", e),
            })
    })
}

// ── Unified Handlers (extract from UnifiedState) ────────────────────

// OTP handlers — delegate to OTP service inside UnifiedState

async fn u_otp_generate(
    State(state): State<UnifiedState>,
) -> Result<Json<OtpResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = state.otp.lock().unwrap();
    match svc.generate_otp() {
        Ok(record) => Ok(Json(OtpResponse {
            otp: record.otp,
            source_number: record.source_number,
            timestamp: record.created_at,
        })),
        Err(msg) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse { error: msg }),
        )),
    }
}

async fn u_otp_history(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.otp.lock().unwrap();
    let history: Vec<_> = svc.otp_history.iter().cloned().collect();
    Json(OtpHistoryResponse { history })
}

async fn u_otp_numbers(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.otp.lock().unwrap();
    let numbers: Vec<u16> = svc.random_numbers.iter().copied().collect();
    let count = numbers.len();
    Json(NumbersResponse { numbers, count })
}

async fn u_otp_params(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.otp.lock().unwrap();
    let params: Vec<String> = svc.sensor_params.iter().cloned().collect();
    let count = params.len();
    Json(ParamsResponse { params, count })
}

async fn u_otp_mac_status(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.otp.lock().unwrap();
    Json(MacStatusResponse {
        mac: svc.mac_address.clone(),
        validated: svc.mac_validated,
        checked: svc.mac_validated.is_some(),
    })
}

// Entropy Engine handlers — delegate to EntropyService inside UnifiedState

async fn u_get_integrity(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.entropy.lock().unwrap();
    let status = svc.get_entropy_integrity();
    Json(IntegrityResponse {
        status: status.to_string(),
        label: format!("Integrity {}", status),
        checked_at: now_iso(),
    })
}

async fn u_get_stats(State(state): State<UnifiedState>) -> impl IntoResponse {
    let svc = state.entropy.lock().unwrap();
    Json(svc.get_entropy_stats())
}

async fn u_get_events(State(state): State<UnifiedState>) -> impl IntoResponse {
    let mut svc = state.entropy.lock().unwrap();
    let events = svc.drain_security_events();
    let normalized: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            serde_json::json!({
                "kind": "event",
                "detail": format!("{:?}", e),
                "timestamp": now_iso(),
            })
        })
        .collect();
    Json(normalized)
}

async fn u_generate_aes_key(
    State(state): State<UnifiedState>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = state.entropy.lock().unwrap();
    match svc.generate_aes_key() {
        Ok(key) => Ok(Json(GeneratedResponse {
            value: key,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn u_generate_password(
    State(state): State<UnifiedState>,
    Json(body): Json<PasswordRequest>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let length = body.length.unwrap_or(24).clamp(12, 96);
    let config = PasswordConfig {
        length,
        ..PasswordConfig::default()
    };
    let mut svc = state.entropy.lock().unwrap();
    match svc.generate_password(config) {
        Ok(pw) => Ok(Json(GeneratedResponse {
            value: pw,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn u_generate_session_token(
    State(state): State<UnifiedState>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = state.entropy.lock().unwrap();
    match svc.generate_session_token() {
        Ok(token) => Ok(Json(GeneratedResponse {
            value: token,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}

async fn u_generate_hex(
    State(state): State<UnifiedState>,
) -> Result<Json<GeneratedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut svc = state.entropy.lock().unwrap();
    match svc.random_hex(32) {
        Ok(hex) => Ok(Json(GeneratedResponse {
            value: hex,
            timestamp: now_iso(),
        })),
        Err(e) => Err(err_response(e)),
    }
}
