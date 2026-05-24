use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, QoS};

use crate::api::EntropyService;
use crate::blockchain::BlockchainValidator;
use crate::otp::OtpService;

#[derive(Debug, Clone)]
pub struct OtpMqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub numbers_topic: String,
    pub params_topic: String,
    pub mac_topic: String,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for OtpMqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".into(),
            broker_port: 1883,
            numbers_topic: "random/numbers".into(),
            params_topic: "random/params".into(),
            mac_topic: "random/MAC".into(),
            client_id: "entropy-vault-otp".into(),
            username: None,
            password: None,
        }
    }
}

pub struct OtpMqttIngestor {
    _handle: thread::JoinHandle<()>,
}

impl OtpMqttIngestor {
    pub fn start(
        config: OtpMqttConfig,
        service: Arc<Mutex<OtpService>>,
        blockchain: Arc<BlockchainValidator>,
        entropy_service: Option<Arc<Mutex<EntropyService>>>,
    ) -> Result<Self, String> {
        let mut mqttoptions = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(15));
        mqttoptions.set_clean_session(true);

        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            mqttoptions.set_credentials(user.clone(), pass.clone());
            log::info!("MQTT credentials set for user '{}'", user);
        }

        let (client, connection) = Client::new(mqttoptions, 256);

        let numbers_topic = config.numbers_topic.clone();
        let params_topic = config.params_topic.clone();
        let mac_topic = config.mac_topic.clone();

        let handle = thread::spawn(move || {
            // Subscribe to all three topics
            if let Err(e) = client.subscribe(&numbers_topic, QoS::AtLeastOnce) {
                log::error!("MQTT subscribe to '{}' failed: {}", numbers_topic, e);
                return;
            }
            if let Err(e) = client.subscribe(&params_topic, QoS::AtLeastOnce) {
                log::error!("MQTT subscribe to '{}' failed: {}", params_topic, e);
                return;
            }
            if let Err(e) = client.subscribe(&mac_topic, QoS::AtLeastOnce) {
                log::error!("MQTT subscribe to '{}' failed: {}", mac_topic, e);
                return;
            }

            log::info!(
                "MQTT subscribed to '{}', '{}', and '{}'",
                numbers_topic,
                params_topic,
                mac_topic,
            );

            Self::run_event_loop(connection, &numbers_topic, &params_topic, &mac_topic, &service, &blockchain, &entropy_service);
        });

        log::info!(
            "OTP MQTT ingestor started: {}:{}",
            config.broker_host,
            config.broker_port,
        );

        Ok(Self { _handle: handle })
    }

    fn run_event_loop(
        mut connection: Connection,
        numbers_topic: &str,
        params_topic: &str,
        mac_topic: &str,
        service: &Arc<Mutex<OtpService>>,
        blockchain: &Arc<BlockchainValidator>,
        entropy_service: &Option<Arc<Mutex<EntropyService>>>,
    ) {
        let mut msg_count: u64 = 0;

        for event in connection.iter() {
            match event {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    let payload = match std::str::from_utf8(&publish.payload) {
                        Ok(s) => s.to_string(),
                        Err(_) => continue,
                    };

                    if let Ok(mut svc) = service.lock() {
                        if publish.topic == numbers_topic {
                            svc.ingest_numbers(&payload);
                            msg_count += 1;
                        } else if publish.topic == params_topic {
                            svc.ingest_params(&payload);
                            msg_count += 1;
                        } else if publish.topic == mac_topic {
                            // Parse JSON: {"mac":"4C:C3:82:36:81:04"}
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&payload) {
                                if let Some(mac) = json.get("mac").and_then(|v| v.as_str()) {
                                    let is_new = svc.ingest_mac(mac);
                                    if is_new {
                                        let skip_blockchain = std::env::var("SKIP_BLOCKCHAIN")
                                            .map(|v| v == "1" || v == "true")
                                            .unwrap_or(false);

                                        if skip_blockchain {
                                            println!("MAC '{}' — blockchain validation skipped (SKIP_BLOCKCHAIN=1)", mac);
                                            svc.set_mac_validated(true);
                                        } else {
                                            println!("Validating MAC '{}' against MultiChain...", mac);
                                            match blockchain.validate_mac(mac) {
                                                Ok(valid) => {
                                                    svc.set_mac_validated(valid);
                                                    if valid {
                                                        println!("MAC '{}' is VALID — accepting random numbers", mac);
                                                    } else {
                                                        println!("MAC '{}' is INVALID — rejecting all random numbers", mac);
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Blockchain validation error: {}", e);
                                                    svc.set_mac_validated(false);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    log::warn!("MAC topic: JSON missing 'mac' field: {}", payload);
                                }
                            } else {
                                log::warn!("MAC topic: invalid JSON: {}", payload);
                            }
                            msg_count += 1;
                        }

                        if msg_count % 100 == 0 {
                            log::debug!(
                                "OTP MQTT: {} messages ingested, {} numbers stored, {} params stored",
                                msg_count,
                                svc.random_numbers.len(),
                                svc.sensor_params.len(),
                            );
                        }
                    }

                    // Feed raw payload bytes into EntropyService (if present)
                    if let Some(ref es) = entropy_service {
                        if let Ok(mut esvc) = es.lock() {
                            esvc.ingest(0x01, publish.payload.as_ref());
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    log::warn!("MQTT connection error (will auto-reconnect): {}", e);
                    std::thread::sleep(Duration::from_secs(2));
                }
            }
        }

        log::info!("OTP MQTT event loop ended after {} messages", msg_count);
    }
}
