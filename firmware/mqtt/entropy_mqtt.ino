/*
 * Entropy Vault — ESP32 MQTT Entropy Publisher
 *
 * Reads analog noise from the ESP32 ADC, mixes it with timing jitter,
 * and publishes raw entropy bytes to an MQTT broker (Eclipse Mosquitto)
 * over WiFi.
 *
 * Hardware: Any ESP32 board (DevKit, Wroom, S3, etc.)
 * Wiring:   Leave an ADC pin floating (or connect to a noise source).
 *           No other wiring needed — uses built-in WiFi.
 *
 * Dependencies (install via Arduino Library Manager):
 *   - PubSubClient by Nick O'Leary (MQTT client)
 *
 * Setup:
 *   1. Install Mosquitto on your computer: brew install mosquitto
 *   2. Start it: mosquitto -d
 *   3. Update WIFI_SSID, WIFI_PASS, MQTT_SERVER below
 *   4. Flash this sketch to your ESP32
 *   5. Run the engine: ENTROPY_MODE=mqtt cargo run
 */

#include <WiFi.h>
#include <PubSubClient.h>

// ---- Configuration ----
const char* WIFI_SSID     = "Navalogy";
const char* WIFI_PASS     = "Man_Mera_Mandir";
const char* MQTT_SERVER   = "10.89.2.96";  // Your Mac's local IP
const int   MQTT_PORT     = 1883;
const char* MQTT_TOPIC    = "entropy-vault/raw";
const char* MQTT_CLIENT   = "entropy-vault-esp32";

// ADC pin left floating for noise (or connect to a noise source).
const int   ADC_PIN       = 34;  // GPIO34 (ADC1_CH6)

// How many entropy bytes to publish per message.
const int   ENTROPY_BYTES = 64;

// Milliseconds between publishes.
const int   PUBLISH_INTERVAL_MS = 500;

// ---- Globals ----
WiFiClient   wifiClient;
PubSubClient mqttClient(wifiClient);

uint32_t entropy_state = 0x6D2B79F5;

// ---- Entropy generation ----

// Mix ADC noise + timing jitter into internal state, produce one byte.
uint8_t entropy_next_byte() {
    uint16_t adc_noise = analogRead(ADC_PIN);
    uint32_t timing    = (uint32_t)(micros() & 0xFFFF);

    uint32_t mixed = entropy_state ^ adc_noise ^ (timing << 11);
    // Avalanche mixing (same as firmware/src/entropy/entropy.c)
    mixed += 0x9E3779B9u;
    mixed ^= mixed >> 16;
    mixed *= 0x85EBCA6Bu;
    mixed ^= mixed >> 13;
    mixed *= 0xC2B2AE35u;
    mixed ^= mixed >> 16;

    entropy_state = mixed;
    return (uint8_t)(mixed & 0xFF);
}

void entropy_fill(uint8_t* buffer, size_t length) {
    for (size_t i = 0; i < length; i++) {
        buffer[i] = entropy_next_byte();
    }
}

// ---- WiFi ----

void wifi_connect() {
    Serial.printf("Connecting to WiFi '%s'...\n", WIFI_SSID);
    WiFi.begin(WIFI_SSID, WIFI_PASS);

    int attempts = 0;
    while (WiFi.status() != WL_CONNECTED && attempts < 30) {
        delay(500);
        Serial.print(".");
        attempts++;
    }

    if (WiFi.status() == WL_CONNECTED) {
        Serial.printf("\nWiFi connected! IP: %s\n", WiFi.localIP().toString().c_str());
    } else {
        Serial.println("\nWiFi connection FAILED. Check SSID/password.");
    }
}

// ---- MQTT ----

void mqtt_reconnect() {
    while (!mqttClient.connected()) {
        Serial.printf("Connecting to MQTT broker %s:%d...\n", MQTT_SERVER, MQTT_PORT);
        if (mqttClient.connect(MQTT_CLIENT)) {
            Serial.println("MQTT connected!");
        } else {
            Serial.printf("MQTT failed (rc=%d). Retrying in 3s...\n", mqttClient.state());
            delay(3000);
        }
    }
}

// ---- Main ----

void setup() {
    Serial.begin(115200);
    delay(1000);

    Serial.println("=== Entropy Vault ESP32 (MQTT Publisher) ===");

    // Seed entropy state from ADC + boot time.
    entropy_state ^= (uint32_t)analogRead(ADC_PIN) ^ ((uint32_t)micros() << 16);

    wifi_connect();

    mqttClient.setServer(MQTT_SERVER, MQTT_PORT);
    mqttClient.setBufferSize(512);
}

void loop() {
    if (WiFi.status() != WL_CONNECTED) {
        wifi_connect();
    }

    if (!mqttClient.connected()) {
        mqtt_reconnect();
    }
    mqttClient.loop();

    // Generate entropy and publish.
    uint8_t buffer[ENTROPY_BYTES];
    entropy_fill(buffer, ENTROPY_BYTES);

    bool ok = mqttClient.publish(MQTT_TOPIC, buffer, ENTROPY_BYTES);
    if (ok) {
        static uint32_t msg_count = 0;
        msg_count++;
        if (msg_count % 20 == 0) {
            Serial.printf("Published %u messages (%u bytes total)\n",
                          msg_count, msg_count * ENTROPY_BYTES);
        }
    } else {
        Serial.println("MQTT publish failed!");
    }

    delay(PUBLISH_INTERVAL_MS);
}
