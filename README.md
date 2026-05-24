---
publishDate: 2026-05-14T00:00:00Z
title: Entropy Vault - Hardware Random Number Generator with Blockchain Device Authentication
excerpt: A multi-sensor hardware entropy collector built on ESP32 and MYOSA, feeding a cryptographic engine over MQTT with blockchain-based device authentication and a real-time dashboard.
image: entropy-vault/cover.jpg
tags:
  - entropy
  - hardware-rng
  - mqtt
  - blockchain
  - esp32
  - myosa
  - cryptography
  - iot-security
---

# Entropy Vault - A Hardware based True Random Number Generator using MYOSA Kit

![IoT](https://img.shields.io/badge/IoT-blue) ![Rust](https://img.shields.io/badge/Rust-orange) ![MOSFET](https://img.shields.io/badge/MOSFET-green) ![MQTT](https://img.shields.io/badge/MQTT-purple) ![MYOSA-kit](https://img.shields.io/badge/MYOSA--kit-red) ![Blockchain](https://img.shields.io/badge/Blockchain-yellow) ![Randomness](https://img.shields.io/badge/Randomness-61DAFB)

> Turn raw sensor noise into cryptographically secure random numbers, verified by blockchain and visualized in real time.

---

## Acknowledgements

Built by **Team TU Ankaja** for the IEEE MYOSA Innovation Challenge. We would like to thank the MYOSA organizers for providing the MYOSA development platform and the opportunity to explore true random number generation. We also acknowledge the guidance and support provided by our mentor **Dr. Rupam Goswami Sir** throughout this project.

---

## Overview

Most random number generators on computers are pseudo-random. They use algorithms that look random but are actually deterministic. For anything security-critical like encryption keys, passwords, or one-time passwords, you need true randomness sourced from physical phenomena.

Entropy Vault solves this by turning the MYOSA sensor board into a dedicated hardware entropy source. It reads analog noise from a MOSFET circuit, combines it with accelerometer jitter, gyroscope drift, temperature fluctuations, ambient light changes, and particle sensor readings. All 21 sensor channels contribute unpredictability. This raw data is sent wirelessly over MQTT to a Rust-based cryptographic engine that conditions, validates, and serves the entropy through an HTTP API. A Next.js dashboard lets you watch everything happen live.

The critical security question: how do you know the data actually came from your hardware and not from a spoofed device? We solve this with a two-layer approach. First, MQTT broker authentication requires credentials. Second, the ESP32 publishes its MAC address on a dedicated MQTT topic, and the backend validates it against a MultiChain blockchain. If the MAC is not registered on-chain, every single random number from that device gets rejected.

**What it does in practice:**

* Collects entropy from 21 physical sensors simultaneously
* Streams data wirelessly to the backend at 500ms intervals via MQTT
* Conditions raw noise through SHA-256 whitening into uniform random bytes
* Feeds a ChaCha20-based DRBG with forward secrecy for cryptographic output
* Runs SP 800-90B health monitoring on every sample (repetition count, adaptive proportion)
* Validates the source device against a blockchain before accepting any data
* Generates AES-256 keys, passwords, session tokens, and hardware-backed OTPs
* Shows all of this on a live dashboard with per-sensor graphs updating every 3 seconds

---

## Demo / Examples

### **Images**

<p align="center">
  <img src="/assets/images/entropy-vault/login-page.jpg" width="800"><br/>
  <i>Login screen for the TU Ankaja dashboard</i>
</p>

<p align="center">
  <img src="/assets/images/entropy-vault/otp-generator.jpg" width="800"><br/>
  <i>OTP Generator page - 6-digit codes derived from hardware random numbers via SHA-256</i>
</p>

<p align="center">
  <img src="/assets/images/entropy-vault/raw-data-viewer.jpg" width="800"><br/>
  <i>Raw Data Viewer showing live sensor graphs - particle concentration, temperature, accelerometer, gyroscope, color sensor, ambient light</i>
</p>

<p align="center">
  <img src="/assets/images/entropy-vault/entropy-engine.jpg" width="800"><br/>
  <i>Entropy Engine dashboard - pool stats, source quality tiers, security event feed, and cryptographic material generation</i>
</p>

<p align="center">
  <img src="/assets/images/entropy-vault/hardware-setup.jpg" width="800"><br/>
  <i>ESP32 with MYOSA sensor board connected over WiFi, publishing entropy to Mosquitto broker</i>
</p>

<p align="center">
  <img src="/assets/images/entropy-vault/blockchain-validation.jpg" width="800"><br/>
  <i>MultiChain MAC validation - terminal showing device authentication against the blockchain</i>
</p>

### **Videos**

<video controls width="100%">
  <source src="/entropy-vault-demo.mp4" type="video/mp4">
</video>

---

## Features (Detailed)

### **1. Analog Noise Generator Circuit (MOSFET)**

Electronic noise in MOSFETs is naturally stochastic, making it an ideal source of true randomness. The circuit uses an IRF540N n-channel MOSFET as the primary noise source:

- **Circuit Design**: The Drain (D) terminal is connected to +3.3V using a 2k ohm pull-up resistor. A random value from 0 to 255 is applied at the Gate (G) from DAC pin 25 of the MYOSA Motherboard through a small 82 ohm gate resistor using `dacWrite(25, esp_random() & 0xFF)`.
- **Output**: The resulting random signal is harvested from the Drain terminal and fed directly to the 12-bit ADC pin 32 of the MYOSA motherboard.

This analog noise is the primary entropy source because electronic noise is physically unpredictable and cannot be reproduced.

### **2. Multi-Sensor Entropy Collection (ESP32 + MYOSA)**

The MYOSA board gives us 21 sensor channels, and we use all of them. The ESP32 reads:

- **MOSFET Noise** - Analog electronic noise from the MOSFET circuit (primary entropy source).
- **Temperature** - Environmental temperature fluctuations add low-frequency entropy.
- **Accelerometer (3-axis)** - Micro-vibrations in X, Y, Z axes. Even a board sitting on a table picks up building vibrations, air conditioning hum, etc.
- **Gyroscope (3-axis)** - Rotational jitter. The sensor noise floor itself contributes randomness.
- **Color Sensor (RGB)** - Raw red, green, blue light readings via the APDS9960 sensor. Ambient light variation adds environmental entropy.
- **Ambient Light** - Lux measurement that changes with any shadow, cloud, or movement nearby.
- **Particle Sensor (PM1.0, PM2.5, PM10)** - Air quality mass concentrations from the PMS5003 sensor. These fluctuate constantly in any real environment.
- **Particle Counts (6 size bins)** - Count of particles above 0.3, 0.5, 1.0, 2.5, 5.0, and 10 micrometers. High-frequency variation.

The ESP32 samples all sensors, packs the readings into CSV, and publishes over MQTT to the `random/params` topic. Simultaneously, 4-digit random numbers derived from MOSFET noise are published to `random/numbers`. The device MAC address is sent as JSON to `random/MAC` for blockchain authentication.

### **3. Wireless Data Pipeline (MQTT)**

We use Eclipse Mosquitto as the MQTT broker. The architecture:

```
ESP32 (MYOSA sensors)
    | WiFi
    v
Mosquitto Broker (laptop, port 1883)
    | 3 topics: random/numbers, random/params, random/MAC
    v
Rust Entropy Engine (subscriber)
    | HTTP API (port 3001)
    v
Next.js Dashboard (port 3000)
```

MQTT gives us reliable, low-latency message delivery. The broker requires username/password authentication, so unauthorized devices cannot connect. QoS level 1 (at least once delivery) ensures no entropy samples are silently dropped.

### **4. Blockchain Device Authentication (MultiChain)**

This is the core security feature. Anyone who knows the MQTT credentials could connect a fake device and publish garbage data. MAC address spoofing is trivial. So we put the trust anchor on a blockchain.

How it works:

1. An administrator registers valid device MAC addresses on a MultiChain blockchain stream called `valid-macs`
2. When an ESP32 connects and publishes to `random/MAC` with `{"mac":"4C:C3:82:36:81:04"}`, the backend receives it
3. The Rust engine calls MultiChain's JSON-RPC API (`liststreamkeyitems`) to check if this MAC exists on-chain
4. If the MAC is found: the device is trusted, all random numbers are accepted
5. If the MAC is NOT found: every single random number from that device is rejected and discarded

The blockchain is immutable. Once a MAC is registered, the record cannot be tampered with. A Docker setup script (`scripts/setup_multichain.sh`) handles the entire MultiChain installation, chain creation, stream setup, and test MAC seeding in one command.

### **5. Cryptographic Entropy Engine (Rust)**

The entropy engine is written in Rust and implements a proper cryptographic pipeline:

- **SHA-256 Whitening** - Raw sensor data is biased (ADC readings cluster, temperature changes slowly). The whitener conditions input through SHA-256 to produce uniformly distributed bytes. Biased input in, uniform output out.
- **Entropy Pool** - A 256-bit SHA-256 accumulation pool. New data is mixed in via `SHA-256(state || input)`. This guarantees entropy only accumulates. Mixing in low-quality data cannot reduce what is already there.
- **Source Quality Tracking** - Each entropy source gets a quality score (min-entropy per byte, confidence, correlation analysis). High-quality sources contribute more to the pool's weighted entropy estimate.
- **SP 800-90B Health Monitoring** - Three continuous health tests run on every sample: repetition count test (catches stuck sensors), adaptive proportion test (catches biased distributions), and entropy degradation check (catches low diversity). A failed health check blocks all output.
- **ChaCha20 DRBG** - A deterministic random bit generator seeded from the entropy pool. Uses ChaCha20 stream cipher for output. After each generation, the key is ratcheted forward so past outputs cannot be recovered even if current state is compromised (forward secrecy). Mandatory reseed after 1 MiB of output.
- **Security Gate** - A policy enforcement layer between the pipeline and output. Production policy requires 256 bits of weighted pool entropy for AES keys, blocks simulated sources, and refuses output during health warnings. Development policy is permissive for testing.

All sensitive state is zeroized on drop using the `zeroize` crate.

### **6. OTP Generation**

One-time passwords are generated by combining a hardware random number with a microsecond timestamp:

```
OTP = SHA-256(random_number_bytes || timestamp_bytes) mod 1,000,000
```

This produces a 6-digit code. The random number comes from MOSFET noise (not pseudo-random), and the timestamp adds uniqueness even if the same number is selected twice. OTP history is maintained for auditing.

### **7. Real-Time Dashboard (Next.js)**

Three pages, each serving a different purpose:

- **OTP Generator** - Generate hardware-backed OTPs with one click. Shows source number, timestamp, and history of past codes.
- **Raw Data Viewer** - Live sensor graphs for all 8 sensor groups, updating every 3 seconds. Particle concentration charts are shown first since they have the most dynamic readings. Raw CSV data and random numbers are displayed below.
- **Entropy Engine** - Full pipeline visibility. Pool entropy bits, source quality tiers, health status, security event feed. Generate AES-256 keys, passwords, and session tokens on demand.

Built with React 19, Recharts for graphing, Framer Motion for animations, Zustand for state management, and Tailwind CSS for styling.

---

## Usage Instructions

### Starting the Full Wireless Pipeline

1. Flash the ESP32 with the MQTT firmware:

```plaintext
cd firmware/mqtt
# Open entropy_mqtt.ino in Arduino IDE
# Update WIFI_SSID, WIFI_PASS, MQTT_SERVER
# Flash to ESP32
```

2. Start the Mosquitto MQTT broker:

```plaintext
mosquitto -d
```

3. Set up MultiChain for device authentication (first time only):

```plaintext
chmod +x scripts/setup_multichain.sh
./scripts/setup_multichain.sh
```

4. Register your ESP32's MAC address on the blockchain:

```plaintext
docker exec entropy-multichain multichain-cli entropy-chain \
  publish valid-macs "4C:C3:82:36:81:04" \
  '{"json":{"mac":"4C:C3:82:36:81:04","status":"valid"}}'
```

5. Start the Rust entropy engine:

```plaintext
./scripts/start-wireless.sh
```

Or manually with environment variables:

```plaintext
cd entropy-engine
ENTROPY_MODE=otp \
  MQTT_HOST=<your-broker-ip> \
  MQTT_PORT=1883 \
  MQTT_USER=<your-mqtt-username> \
  MQTT_PASS=<your-mqtt-password> \
  MULTICHAIN_RPC_URL=http://localhost:6740 \
  MULTICHAIN_RPC_USER=multichainrpc \
  MULTICHAIN_RPC_PASS=<your-rpc-password> \
  cargo run
```

6. Start the frontend dashboard:

```plaintext
cd frontend
npm install
npm run dev
```

7. Open `http://localhost:3000` in your browser.

### Running Without Hardware (Simulation)

If you do not have the ESP32 or MYOSA board, you can test with the simulator:

```plaintext
# Terminal 1: Start Mosquitto
mosquitto -d

# Terminal 2: Run the MQTT simulator
python3 scripts/mqtt_simulator.py

# Terminal 3: Start the engine in MQTT mode
cd entropy-engine
ENTROPY_MODE=mqtt cargo run

# Terminal 4: Start the dashboard
cd frontend && npm run dev
```

### Running the Firmware Simulator (No WiFi needed)

```plaintext
cd firmware
make run
```

This compiles and runs the C entropy simulator locally, printing a sample of generated entropy bytes.

---

## Tech Stack

* **ESP32 (MYOSA Motherboard)** - Microcontroller with built-in WiFi for wireless sensor data transmission
* **MYOSA Sensor Board** - 21-channel multi-sensor board (MOSFET noise, IMU, color, light, particle sensor)
* **IRF540N MOSFET** - N-channel MOSFET for analog noise generation (primary entropy source)
* **APDS9960** - RGB and ambient light sensor module
* **PMS5003** - Particle matter sensor for air quality readings
* **C** - Firmware for ADC noise reading and entropy mixing with avalanche function
* **Arduino (PubSubClient)** - MQTT client library for ESP32 WiFi publishing
* **Rust** - Entropy engine with SHA-256 whitening, ChaCha20 DRBG, health monitoring, and HTTP API
* **rumqttc** - Rust MQTT client for subscribing to broker topics
* **axum** - Rust HTTP framework serving the REST API
* **Eclipse Mosquitto** - MQTT broker with username/password authentication
* **MultiChain** - Private blockchain for device MAC address validation (Docker)
* **Next.js 16** - React framework for the dashboard frontend
* **Recharts** - Charting library for real-time sensor data visualization
* **Framer Motion** - Animation library for smooth UI transitions
* **Zustand** - Lightweight state management for React
* **Tailwind CSS v4** - Utility-first CSS framework
---

## Requirements / Installation

### Hardware Requirements

- MYOSA Motherboard (ESP32-based)
- MYOSA OLED Display
- MYOSA Accelerometer/Gyroscope Module
- MYOSA Light/Proximity Module (APDS9960)
- Particle Sensor (PMS5003)
- IRF540N n-channel MOSFET
- 2k ohm pull-up resistor, 82 ohm gate resistor
- Breadboard and jumper wires

### Software Requirements

- Rust toolchain (1.70+)
- Node.js (18+)
- Python 3 (for MQTT simulator)
- Docker Desktop (for MultiChain blockchain)
- Arduino IDE (for flashing ESP32)

### Dependencies

Rust engine:

```plaintext
cd entropy-engine
cargo build
```

Frontend:

```plaintext
cd frontend
npm install
```

Python simulator:

```plaintext
pip install paho-mqtt
```

MQTT broker:

```plaintext
brew install mosquitto
```

### Quick Check (All Components)

```plaintext
./scripts/check-all.sh
```

This runs the firmware simulator, Rust tests (96 tests), and frontend type checking.

---

## File Structure

```
/entropy-vault
  ├── firmware/
  │   ├── src/
  │   │   ├── main.c              # Firmware simulator entry point
  │   │   ├── adc/adc.c           # ADC noise reader (xorshift simulation)
  │   │   ├── sensors/sensors.c   # Timing jitter sensor
  │   │   ├── entropy/entropy.c   # Avalanche mixing function
  │   │   └── uart/uart.c         # UART hex output
  │   ├── include/entropy_vault.h # Shared header
  │   ├── mqtt/entropy_mqtt.ino   # ESP32 MQTT publisher (Arduino)
  │   └── Makefile
  │
  ├── entropy-engine/
  │   ├── src/
  │   │   ├── main.rs             # Entry point (serial/mqtt/otp modes)
  │   │   ├── lib.rs              # Module declarations
  │   │   ├── api/mod.rs          # EntropyService API
  │   │   ├── whitening/mod.rs    # SHA-256 entropy conditioning
  │   │   ├── pool/mod.rs         # Entropy accumulation pool
  │   │   ├── drbg/mod.rs         # ChaCha20 DRBG with forward secrecy
  │   │   ├── health/mod.rs       # SP 800-90B health monitoring
  │   │   ├── quality/mod.rs      # Source quality tracking
  │   │   ├── security/mod.rs     # Security gate and policy enforcement
  │   │   ├── crypto/mod.rs       # Password/key generation with rejection sampling
  │   │   ├── mqtt/mod.rs         # MQTT entropy ingestor
  │   │   ├── otp/mod.rs          # OTP generation service
  │   │   ├── otp_mqtt/mod.rs     # OTP MQTT ingestor (3 topics)
  │   │   ├── blockchain/mod.rs   # MultiChain MAC validation
  │   │   ├── server/mod.rs       # HTTP API server (axum)
  │   │   ├── parser/mod.rs       # Binary protocol parser
  │   │   ├── serial/mod.rs       # Serial port source
  │   │   ├── models/mod.rs       # Data models
  │   │   └── errors/mod.rs       # Error types
  │   ├── tests/engine.rs         # Integration tests (26 tests)
  │   └── Cargo.toml
  │
  ├── frontend/
  │   ├── app/
  │   │   ├── page.tsx            # Login page
  │   │   ├── otp/page.tsx        # OTP Generator page
  │   │   ├── data/page.tsx       # Raw Data Viewer (sensor graphs)
  │   │   ├── entropy/page.tsx    # Entropy Engine dashboard
  │   │   └── layout.tsx          # Root layout with navbar
  │   ├── components/             # UI components (NavBar, GeneratePanel, etc.)
  │   ├── services/               # API service functions
  │   ├── store/                  # Zustand state stores
  │   ├── types/                  # TypeScript type definitions
  │   └── package.json
  │
  ├── scripts/
  │   ├── start-wireless.sh       # Start full wireless pipeline
  │   ├── setup_multichain.sh     # Docker MultiChain setup
  │   ├── blockchain              # Blockchain CLI helper
  │   ├── demo-blockchain.sh      # Blockchain video demo
  │   ├── mqtt_simulator.py       # MQTT entropy simulator (no hardware)
  │   ├── otp_simulator.py        # OTP MQTT simulator (3 topics)
  │   ├── check-all.sh            # Run all local checks
  │   ├── flash-esp32.sh          # ESP32 flashing helper
  │   └── start_all.sh            # Start all services
  │
  └── LICENSE                     # MIT License
```

---

## License

MIT License. See the LICENSE file for full text.

---

## Contribution Notes

This project is open source. If you want to contribute:

1. Fork the repository
2. Create a feature branch
3. Run `./scripts/check-all.sh` to make sure everything passes
4. Submit a pull request

Areas where contributions would be useful:

- Adding TLS/mTLS support for encrypted MQTT connections
- Implementing HMAC challenge-response for stronger device authentication
- Adding more entropy health tests (e.g., NIST SP 800-22 test suite)
- Mobile app for monitoring the dashboard remotely
