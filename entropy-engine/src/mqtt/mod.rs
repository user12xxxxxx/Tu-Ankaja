use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, QoS};

use crate::api::EntropyService;
use crate::errors::EntropyError;

/// Configuration for connecting to an MQTT broker (e.g. Eclipse Mosquitto).
#[derive(Debug, Clone)]
pub struct MqttConfig {
    /// Broker hostname or IP address.
    pub broker_host: String,
    /// Broker port (default: 1883 for unencrypted MQTT).
    pub broker_port: u16,
    /// MQTT topic where entropy bytes are published.
    pub topic: String,
    /// Source ID for entropy tracking (matches firmware source_id).
    pub source_id: u8,
    /// Unique client ID for this MQTT connection.
    pub client_id: String,
    /// Optional MQTT username for broker authentication.
    pub username: Option<String>,
    /// Optional MQTT password for broker authentication.
    pub password: Option<String>,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: "localhost".into(),
            broker_port: 1883,
            topic: "entropy-vault/raw".into(),
            source_id: 0x01,
            client_id: "entropy-vault-engine".into(),
            username: None,
            password: None,
        }
    }
}

/// MQTT entropy ingestor — subscribes to a broker topic and pushes received
/// entropy bytes into an `EntropyService` running in push mode.
///
/// # Architecture
///
/// ```text
/// Hardware Board (ESP32)
///     | [WiFi]
///     v
/// Mosquitto Broker (localhost:1883)
///     | topic: "entropy-vault/raw"
///     v
/// MqttIngestor (background thread)
///     | calls service.ingest(source_id, bytes)
///     v
/// EntropyService (push mode)
///     | pool already fed → generate output on demand
/// ```
///
/// The ingestor runs a background thread that blocks on the MQTT event loop.
/// Each incoming publish message is forwarded to the shared `EntropyService`
/// via `ingest()`, which runs health checks and mixes into the entropy pool.
pub struct MqttIngestor {
    _handle: thread::JoinHandle<()>,
}

impl MqttIngestor {
    /// Connect to the MQTT broker and start ingesting entropy in a background thread.
    ///
    /// The `service` must be created with `EntropyService::new_push()` so it
    /// operates in push mode (no serial source, pool fed externally).
    ///
    /// Returns immediately after spawning the background thread. The thread
    /// runs until the broker connection is permanently lost.
    pub fn start(
        config: MqttConfig,
        service: Arc<Mutex<EntropyService>>,
    ) -> Result<Self, EntropyError> {
        let mut mqttoptions = MqttOptions::new(
            &config.client_id,
            &config.broker_host,
            config.broker_port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            mqttoptions.set_credentials(user.clone(), pass.clone());
            log::info!("MQTT credentials set for user '{}'", user);
        }

        let (client, connection) = Client::new(mqttoptions, 256);

        let topic = config.topic.clone();
        let source_id = config.source_id;

        let handle = thread::spawn(move || {
            if let Err(e) = client.subscribe(&topic, QoS::AtLeastOnce) {
                log::error!("MQTT subscribe to '{}' failed: {}", topic, e);
                return;
            }
            log::info!("MQTT subscribed to '{}', waiting for entropy...", topic);

            Self::run_event_loop(connection, source_id, &service);
        });

        log::info!(
            "MQTT ingestor started: {}:{} topic='{}'",
            config.broker_host,
            config.broker_port,
            config.topic,
        );

        Ok(Self { _handle: handle })
    }

    /// Process MQTT events forever, pushing entropy bytes into the service.
    fn run_event_loop(
        mut connection: Connection,
        source_id: u8,
        service: &Arc<Mutex<EntropyService>>,
    ) {
        let mut total_bytes: u64 = 0;

        for event in connection.iter() {
            match event {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    let payload = &publish.payload;
                    if payload.is_empty() {
                        continue;
                    }

                    if let Ok(mut svc) = service.lock() {
                        svc.ingest(source_id, payload);
                        total_bytes += payload.len() as u64;

                        if total_bytes % 1024 < payload.len() as u64 {
                            log::debug!(
                                "MQTT ingested {} total bytes from source 0x{:02X}",
                                total_bytes,
                                source_id,
                            );
                        }
                    }
                }
                Ok(_) => {
                    // ConnAck, SubAck, PingResp — normal protocol traffic.
                }
                Err(e) => {
                    log::warn!("MQTT connection error (will auto-reconnect): {}", e);
                }
            }
        }

        log::info!("MQTT event loop ended after {} bytes ingested", total_bytes);
    }
}
