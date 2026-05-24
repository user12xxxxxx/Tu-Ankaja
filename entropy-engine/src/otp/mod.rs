use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_NUMBERS: usize = 10_000;
const MAX_PARAMS: usize = 1_000;
const MAX_HISTORY: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct OtpRecord {
    pub otp: String,
    pub source_number: u16,
    pub timestamp_micros: u128,
    pub created_at: String,
}

pub struct OtpService {
    pub random_numbers: VecDeque<u16>,
    pub sensor_params: VecDeque<String>,
    pub otp_history: VecDeque<OtpRecord>,
    /// The first MAC address extracted from random/numbers CSV.
    pub mac_address: Option<String>,
    /// Whether the MAC has been validated against the blockchain.
    /// None = not yet checked, Some(true) = valid, Some(false) = invalid.
    pub mac_validated: Option<bool>,
}

impl OtpService {
    pub fn new() -> Self {
        Self {
            random_numbers: VecDeque::new(),
            sensor_params: VecDeque::new(),
            otp_history: VecDeque::new(),
            mac_address: None,
            mac_validated: None,
        }
    }

    /// Parse CSV payload like "4716,5636,2202,5006".
    /// Only stores numbers if MAC validation has passed (or not yet checked).
    pub fn ingest_numbers(&mut self, csv: &str) {
        for token in csv.split(',') {
            let trimmed = token.trim();

            // If MAC was checked and is invalid, reject all numbers
            if self.mac_validated == Some(false) {
                continue;
            }

            if let Ok(num) = trimmed.parse::<u16>() {
                if num <= 9999 {
                    self.random_numbers.push_back(num);
                    while self.random_numbers.len() > MAX_NUMBERS {
                        self.random_numbers.pop_front();
                    }
                }
            }
        }
    }

    /// Store a MAC address received from the dedicated random/MAC topic.
    /// Only stores the first MAC seen — subsequent MACs are ignored.
    /// Returns true if this was a new MAC (needs blockchain validation).
    pub fn ingest_mac(&mut self, mac: &str) -> bool {
        if self.mac_address.is_some() {
            return false;
        }
        self.mac_address = Some(mac.to_string());
        log::info!("MAC address received: {}", mac);
        true
    }

    /// Set the MAC validation result from blockchain check.
    pub fn set_mac_validated(&mut self, valid: bool) {
        self.mac_validated = Some(valid);
        if !valid {
            log::warn!("MAC validation failed — clearing all stored numbers");
            self.random_numbers.clear();
        } else {
            log::info!("MAC validated successfully against blockchain");
        }
    }

    /// Store raw sensor parameter string.
    pub fn ingest_params(&mut self, raw: &str) {
        if raw.is_empty() {
            return;
        }
        self.sensor_params.push_back(raw.to_string());
        while self.sensor_params.len() > MAX_PARAMS {
            self.sensor_params.pop_front();
        }
    }

    /// Generate a 6-digit OTP from a random 4-digit number + microsecond timestamp.
    pub fn generate_otp(&mut self) -> Result<OtpRecord, String> {
        if self.random_numbers.is_empty() {
            return Err("No random numbers available yet. Wait for MQTT data.".into());
        }

        let timestamp_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();

        // Pick a number using timestamp as index
        let index = (timestamp_micros as usize) % self.random_numbers.len();
        let source_number = self.random_numbers[index];

        // SHA-256(number_bytes || timestamp_bytes) → 6-digit OTP
        let mut hasher = Sha256::new();
        hasher.update(source_number.to_be_bytes());
        hasher.update(timestamp_micros.to_be_bytes());
        let digest = hasher.finalize();

        let raw_value = u32::from_be_bytes([digest[28], digest[29], digest[30], digest[31]]);
        let otp_num = raw_value % 1_000_000;
        let otp = format!("{:06}", otp_num);

        let record = OtpRecord {
            otp,
            source_number,
            timestamp_micros,
            created_at: now_iso(),
        };

        self.otp_history.push_front(record.clone());
        while self.otp_history.len() > MAX_HISTORY {
            self.otp_history.pop_back();
        }

        Ok(record)
    }
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO-ish timestamp
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let s = secs % 60;
    format!(
        "2025-01-01T{:02}:{:02}:{:02}Z",
        hours, mins, s
    )
}
