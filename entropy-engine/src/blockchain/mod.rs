use serde_json::Value;

/// Validates MAC addresses against a MultiChain blockchain stream.
///
/// MultiChain stores valid MAC addresses in a stream called "valid-macs".
/// Each valid MAC is published as a stream key. This validator queries
/// the MultiChain JSON-RPC API to check if a given MAC exists.
pub struct BlockchainValidator {
    rpc_url: String,
    rpc_user: String,
    rpc_password: String,
    stream_name: String,
}

impl BlockchainValidator {
    pub fn new(rpc_url: String, rpc_user: String, rpc_password: String) -> Self {
        Self {
            rpc_url,
            rpc_user,
            rpc_password,
            stream_name: "valid-macs".into(),
        }
    }

    /// Create a validator from environment variables.
    /// Falls back to sensible defaults for local development.
    pub fn from_env() -> Self {
        let rpc_url = std::env::var("MULTICHAIN_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:6740".into());
        let rpc_user = std::env::var("MULTICHAIN_RPC_USER")
            .unwrap_or_else(|_| "multichainrpc".into());
        let rpc_password = std::env::var("MULTICHAIN_RPC_PASS")
            .unwrap_or_default();

        Self::new(rpc_url, rpc_user, rpc_password)
    }

    /// Validate a MAC address against the MultiChain "valid-macs" stream.
    ///
    /// Calls `liststreamkeyitems` via JSON-RPC. If the stream contains
    /// entries with this MAC as a key, the MAC is considered valid.
    pub fn validate_mac(&self, mac: &str) -> Result<bool, String> {
        let payload = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "mac-check",
            "method": "liststreamkeyitems",
            "params": [&self.stream_name, mac]
        });

        log::info!(
            "Validating MAC '{}' against blockchain at {}",
            mac,
            self.rpc_url
        );

        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize RPC payload: {}", e))?;

        let auth_header = format!(
            "Basic {}",
            base64_encode(&format!("{}:{}", self.rpc_user, self.rpc_password))
        );

        let response = ureq::post(&self.rpc_url)
            .set("Content-Type", "application/json")
            .set("Authorization", &auth_header)
            .send_bytes(payload_str.as_bytes())
            .map_err(|e| format!("MultiChain RPC request failed: {}", e))?;

        let body_str = response
            .into_string()
            .map_err(|e| format!("Failed to read MultiChain response: {}", e))?;

        let body: Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse MultiChain response: {}", e))?;

        // Check if "result" is a non-empty array
        if let Some(result) = body.get("result") {
            if let Some(arr) = result.as_array() {
                let valid = !arr.is_empty();
                log::info!(
                    "MAC '{}' blockchain validation: {} ({} entries found)",
                    mac,
                    if valid { "VALID" } else { "INVALID" },
                    arr.len()
                );
                return Ok(valid);
            }
        }

        // Check for RPC error
        if let Some(error) = body.get("error") {
            if !error.is_null() {
                let msg = format!("MultiChain RPC error: {}", error);
                log::error!("{}", msg);
                return Err(msg);
            }
        }

        // Empty result means MAC not found
        Ok(false)
    }
}

/// Simple base64 encoding for HTTP Basic Auth (no external dependency needed).
fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}
