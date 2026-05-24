---
publishDate: 2026-05-14T00:00:00Z
title: TU Ankaja - Hardware Random Number Generator with Blockchain Device Authentication
excerpt: A multi-sensor hardware entropy collector built on ESP32 and MYOSA, feeding a cryptographic engine over MQTT with blockchain-based device authentication and a real-time dashboard.
image: assets/box-exterior-front.png
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

# TU Ankaja - A Hardware based True Random Number Generator using MYOSA Kit

![IoT](https://img.shields.io/badge/IoT-blue) ![Rust](https://img.shields.io/badge/Rust-orange) ![MOSFET](https://img.shields.io/badge/MOSFET-green) ![MQTT](https://img.shields.io/badge/MQTT-purple) ![MYOSA-kit](https://img.shields.io/badge/MYOSA--kit-red) ![Blockchain](https://img.shields.io/badge/Blockchain-yellow) ![Randomness](https://img.shields.io/badge/Randomness-61DAFB)

> Turn raw sensor noise into cryptographically secure random numbers, verified by blockchain and visualized in real time.

---

## Team Members

| Name | Department | Role |
|------|-----------|------|
| Yash Sharma | B.Tech 4th Semester, ECE | Lead Developer — Rust backend, MQTT pipeline, blockchain integration, frontend dashboard |
| Nautesh Kanojiya | B.Tech 4th Semester, ECE | Hardware Design — MOSFET noise circuit, chaotic box construction, sensor wiring |
| Nabjyoti | B.Tech 4th Semester, ECE | Firmware — ESP32 MQTT publisher, sensor data collection, BCD clamping logic |
| Hritima Rabha | B.Tech 4th Semester, ECE | Testing & Documentation — Integration tests, README, demo scripts |

## Acknowledgements

Built by **Team TU Ankaja** for the IEEE MYOSA Innovation Challenge, organized by the **IEEE Sensors Council**. We would like to thank the MYOSA organizers and IEEE Sensors Council for providing the MYOSA development platform and the opportunity to explore true random number generation. We also acknowledge the guidance and support provided by our mentor **Dr. Rupam Goswami**, Professor, Department of ECE, Tezpur University, throughout this project.

---

## Overview

**TU Ankaja** is an innovative hardware-software system designed to generate true random numbers by combining natural stochastic electronic noise with unpredictable physical parameters.

**What problem does it solve?**
- Overcomes the predictability of purely algorithmic pseudo-random number generators.
- Provides a **low analog computational cost** solution for capturing random electronic fluctuations.
- Serves as a customized hardware source for true randomness, perfectly **tailored for low-to-moderate priority security applications.**

**Key Features:**
* **Analog Noise Generation:** Utilizes an IRF540N n-channel MOSFET as a switch to generate high-frequency noise signals.
* **Multi-Sensor Aggregation:** Captures physical parameters like electronic noise using MOSFET, RGB light, ambient light, temperature, gyroscope data (in x, y, z), air particles simultaneously.
* **Wireless Data Pipeline:** Streams all 21 sensor channels over MQTT to a Rust-based cryptographic engine at 500ms intervals.
* **Blockchain Device Authentication:** Validates the ESP32 MAC address against a MultiChain blockchain before accepting any data.
* **Cryptographic Engine:** SHA-256 whitening, ChaCha20 DRBG with forward secrecy, SP 800-90B health monitoring.
* **OTP Generation:** Generates hardware-backed 6-digit one-time passwords from MOSFET noise + SHA-256.
* **Real-Time Dashboard:** Live sensor graphs, entropy pool stats, and cryptographic material generation on a Next.js frontend.

---

## Cover Image

<p align="center">
  <img src="assets/box-exterior-front.png" width="500"><br/>
  <i>TU Ankaja — the chaotic box exterior. Label reads "TU Ankaja, Tezpur University — IEEE MYOSA 5.0"</i>
</p>

---

## System Block Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CHAOTIC BOX (45×45 cm)                       │
│                                                                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ IRF540N  │  │ APDS9960 │  │ PMS5003  │  │ MPU6050 (Accel/  │   │
│  │ MOSFET   │  │ RGB+Light│  │ Particle │  │ Gyro) + BMP180   │   │
│  │ Noise    │  │ Sensor   │  │ Sensor   │  │ Temp/Pressure    │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬──────────┘   │
│       │ ADC          │ I2C         │ UART            │ I2C          │
│       └──────────────┴─────────────┴────────────────┘              │
│                              │                                      │
│                    ┌─────────┴─────────┐                           │
│                    │  ESP32 (MYOSA     │                           │
│                    │  Motherboard)     │                           │
│                    │  21 sensor channels│                          │
│                    └─────────┬─────────┘                           │
└──────────────────────────────┼──────────────────────────────────────┘
                               │ WiFi (MQTT, 500ms)
                               ▼
                    ┌─────────────────────┐
                    │  Mosquitto Broker   │
                    │  Port 1883          │
                    │  (Auth: user/pass)  │
                    └──────┬──────────────┘
                           │ 3 Topics:
                           │  random/numbers
                           │  random/params
                           │  random/MAC
                           ▼
              ┌────────────────────────────┐
              │  Rust Backend              │
              │  ┌──────────────────────┐  │
              │  │ Blockchain Validator │  │◄──── MultiChain (Docker)
              │  │ (MAC check via RPC)  │  │      valid-macs stream
              │  └──────────┬───────────┘  │
              │             ▼              │
              │  ┌──────────────────────┐  │
              │  │ SHA-256 Whitening    │  │
              │  └──────────┬───────────┘  │
              │             ▼              │
              │  ┌──────────────────────┐  │
              │  │ Entropy Pool (256-bit)│ │
              │  │ + SP 800-90B Health  │  │
              │  └──────────┬───────────┘  │
              │             ▼              │
              │  ┌──────────────────────┐  │
              │  │ ChaCha20 DRBG       │  │
              │  │ (Forward Secrecy)   │  │
              │  └──────────┬───────────┘  │
              │             ▼              │
              │  ┌──────────────────────┐  │
              │  │ Security Gate       │  │
              │  │ OTP / AES Key Gen   │  │
              │  └──────────┬───────────┘  │
              │             │ HTTP API     │
              └─────────────┼──────────────┘
                            │ Port 3001
                            ▼
              ┌────────────────────────────┐
              │  Next.js Dashboard         │
              │  Port 3000                 │
              │  ┌────┐ ┌────┐ ┌────────┐ │
              │  │OTP │ │Raw │ │Entropy │ │
              │  │Gen │ │Data│ │Engine  │ │
              │  └────┘ └────┘ └────────┘ │
              └────────────────────────────┘
```

---

## Demo / Examples

### **Fig 1. Hardware — Exterior & Design**

<p align="center">
  <img src="assets/box-exterior-side.png" width="400"><br/>
  <i>Fig 1a: Side view with door open — mirrored interior walls visible, sensors and wiring inside</i>
</p>

<p align="center">
  <img src="assets/box-dimensions.jpg" width="400"><br/>
  <i>Fig 1b: Hand-drawn design sketch — box dimensions: 18×18×25 cm, base: 20×20×2.5 cm</i>
</p>

### **Fig 2. Hardware — MOSFET Noise Circuit**

<p align="center">
  <img src="assets/mosfet-circuit-closeup.jpeg" width="600"><br/>
  <i>Fig 2: Close-up of the MOSFET noise circuit on perfboard (2kΩ pull-up + 82Ω gate resistor) mounted on the MYOSA motherboard with 0.96" OLED display</i>
</p>

### **Fig 3. Hardware — Inside the Chaotic Box**

<p align="center">
  <img src="assets/chaotic-box-inside-full.jpeg" width="600"><br/>
  <i>Fig 3a: Inside the chaotic box — mirrored foil walls, motor shaft with spinning disc, PMS5003 particle sensor (blue, top-left), MYOSA modules (red PCBs)</i>
</p>

<p align="center">
  <img src="assets/chaotic-box-fan.jpeg" width="600"><br/>
  <i>Fig 3b: Fan blade for particle agitation — blows air across the PMS5003 sensor to generate unpredictable particle readings</i>
</p>

<p align="center">
  <img src="assets/chaotic-box-sensors.jpeg" width="600"><br/>
  <i>Fig 3c: MYOSA Light/Proximity module (APDS9960, red PCB on wall), motor shaft with LED disc, and sensor wiring harness</i>
</p>

<p align="center">
  <img src="assets/chaotic-box-bottom.jpeg" width="600"><br/>
  <i>Fig 3d: Bottom of chaotic box — MYOSA accelerometer/gyroscope module, BMP180 temperature sensor (blue PCB), and MOSFET perfboard with resistors</i>
</p>

<p align="center">
  <img src="assets/chaotic-box-motor.jpeg" width="600"><br/>
  <i>Fig 3e: Motor assembly with rotating disc and colored LEDs for visual disturbance of the APDS9960 sensor</i>
</p>

### **Fig 4. Software — Dashboard**

> Screenshots of the dashboard pages (login, OTP generator, raw data viewer, cryptographic dashboard) are shown during the video demo.

### **Video Demo**

> Demo video showing the full pipeline: ESP32 publishing sensor data → MQTT broker → Rust engine → blockchain MAC validation → live dashboard

---

## Features (Detailed)

### **1. Analog Noise Generator Circuit**

Electronic noise in MOSFETs is naturally stochastic. Circuit Design: The Drain (D) terminal is connected to +3.3V using a 2k ohm pull-up resistor. Input: A random value from 0 to 255 using `dacWrite(25, esp_random() & 0xFF)` function is applied at the Gate (G) from the DAC pin 25 of MYOSA Motherboard through a small gate resistor 82 ohm. Output: The resulting random signal is harvested from the Drain terminal and fed directly to the 12-bit ADC pin 32 of the MYOSA motherboard.

```
    +3.3V
      │
     [2kΩ]  ← Pull-up resistor
      │
      ├──────── ADC Pin 32 (MYOSA) ← Output: random noise signal
      │
    Drain
      │
   ┌──┴──┐
   │IRF540N│  ← N-channel MOSFET
   └──┬──┘
    Gate
      │
     [82Ω] ← Gate resistor
      │
    DAC Pin 25 (MYOSA) ← Input: dacWrite(25, esp_random() & 0xFF)
      │
    Source
      │
     GND
```

**ESP32 Project Pinout:**

| Component / Function | ESP32 Pin |
|---|---|
| DAC output / MOSFET Gate input | GPIO 25 |
| ADC input / MOSFET Drain output | GPIO 32 |
| Motor Driver ENB | GPIO 23 |
| Motor Driver IN3 | GPIO 26 |
| Motor Driver IN4 | GPIO 27 |
| LED 1 | GPIO 5 |
| LED 2 | GPIO 18 |
| LED 3 | GPIO 19 |
| UART Tx (PMS5003) | GPIO 16 |
| UART Rx (PMS5003) | GPIO 17 |
| I2C SDA | GPIO 21 |
| I2C SCL | GPIO 22 |

### **2. Chaotic Hardware Environment**

To gather unpredictable digital data, a 45 cm x 45 cm box with a rough mirrored inner wall houses multiple stimuli:

- **Visual Disturbance:** A motor rotates colored LEDs and sweeps a disc around the APDS9960 sensor to trigger random RGB and gesture data.
- **Particle Agitation:** A PC fan continuously blows air inside the box, scattering particles for the PMS5003 sensor to detect.
- **Environmental Metrics:** External BMP180 and CCS811 sensors gather ambient temperature, pressure, humidity, and volatile organic compounds to add extra environmental entropy.

### **3. Digital Processing & BCD Clamping**

- **Data Collection:** Sensors communicate via I2C and UART (PMS5003), sending 8-bit digital data packets.
- **Array Initialization:** The system accumulates two sets of 8-bit data into a 16-bit variable for each sensor.
- **Random Bit Selection:** A software "BitPicker" randomly selects bits from across the sensor arrays.
- **BCD Clamping:** The 16 random bits are grouped, and a modulo operator (%10) limits the decimal equivalent of the chunks to 9 (preventing hex values up to 15), finalizing the 16-bit random output.

### **4. Wireless Data Pipeline (MQTT)**

We use Eclipse Mosquitto as the MQTT broker. The ESP32 samples all 21 sensors, packs readings into CSV, and publishes over WiFi:

```
ESP32 (MYOSA sensors)
    | WiFi
    v
Mosquitto Broker (laptop, port 1883)
    | 3 topics: random/numbers, random/params, random/MAC
    v
Rust Backend (subscriber)
    | HTTP API (port 3001)
    v
Next.js Dashboard (port 3000)
```

MQTT gives us reliable, low-latency message delivery. The broker requires username/password authentication, so unauthorized devices cannot connect. QoS level 1 ensures no entropy samples are silently dropped.

### **5. Blockchain Device Authentication (MultiChain)**

This is the core security feature. Anyone who knows the MQTT credentials could connect a fake device and publish garbage data. So we put the trust anchor on a blockchain.

1. An administrator registers valid device MAC addresses on a MultiChain blockchain stream called `valid-macs`
2. When an ESP32 connects and publishes to `random/MAC` with `{"mac":"4C:C3:82:36:81:04"}`, the backend receives it
3. The Rust engine calls MultiChain's JSON-RPC API (`liststreamkeyitems`) to check if this MAC exists on-chain
4. If the MAC is found: the device is trusted, all random numbers are accepted
5. If the MAC is NOT found: every single random number from that device is rejected and discarded

The blockchain is immutable. Once a MAC is registered, the record cannot be tampered with.

### **6. OTP Generation**

One-time passwords are generated by combining a hardware random number with a microsecond timestamp:

```
OTP = SHA-256(random_number_bytes || timestamp_bytes) mod 1,000,000
```

This produces a 6-digit code. The random number comes from MOSFET noise (not pseudo-random), and the timestamp adds uniqueness even if the same number appears twice.

### **7. Real-Time Dashboard (Next.js)**

Three pages, each serving a different purpose:

- **OTP Generator** - Generate hardware-backed OTPs with one click. Shows source number, timestamp, and history.
- **Raw Data Viewer** - Live sensor graphs for all 8 sensor groups, updating every 3 seconds.
- **Cryptographic Dashboard** - Pool entropy bits, source quality tiers, health status, security event feed. Generate AES-256 keys, passwords, and session tokens on demand.

---

## Entropy Source Quality Tiers

The engine classifies each sensor source into quality tiers based on entropy contribution and health test pass rate:

| Tier | Entropy Bits | Health Pass Rate | Sources |
|------|-------------|------------------|---------|
| **Excellent** | ≥ 7.5 bits/byte | > 99% | MOSFET noise (primary) |
| **Good** | 5.0 – 7.5 bits/byte | > 95% | Accelerometer, Gyroscope, Particle sensor |
| **Fair** | 2.0 – 5.0 bits/byte | > 90% | RGB color, Ambient light, Temperature |
| **Poor** | < 2.0 bits/byte | < 90% | Rejected — not mixed into pool |

Sources classified as **Poor** are flagged in the security event feed and excluded from the entropy pool. The dashboard displays real-time tier assignments for all active sources.

---

## MYOSA Libraries & Modules Used

This project uses the following MYOSA-provided modules and libraries:

| MYOSA Module | Library / Interface | Purpose |
|---|---|---|
| MYOSA Motherboard (ESP32) | `WiFi.h`, `PubSubClient.h`, DAC (`dacWrite`), ADC (`analogRead`) | WiFi connectivity, MQTT publishing, MOSFET gate drive, noise sampling |
| MYOSA Accelerometer/Gyroscope | `Wire.h` (I2C, address `0x68`) | 6-axis motion data (accel x/y/z, gyro x/y/z) for entropy mixing |
| MYOSA Light/Proximity (APDS9960) | `SparkFun_APDS9960.h` (I2C) | RGB color values and ambient light intensity |
| MYOSA OLED Display | `Adafruit_SSD1306.h` (I2C) | On-device status display |
| PMS5003 Particle Sensor | UART (`Serial2`) | PM1.0, PM2.5, PM10 particle concentration readings |
| BMP180 | `Adafruit_BMP085.h` (I2C) | Temperature and barometric pressure |

> **Note:** The PMS5003 and BMP180 are external sensors not included in the standard MYOSA kit. They were added to increase the number of independent entropy sources from 4 to 8 sensor groups (21 total channels).

---

## Hardware Deviations from Original MYOSA Kit

| Change | Reason |
|--------|--------|
| Added external IRF540N MOSFET circuit on breadboard | MYOSA kit does not include a dedicated analog noise source. The MOSFET's stochastic drain noise provides the primary entropy source with ~7.8 bits/byte. |
| Added PMS5003 particle sensor via UART | Increases entropy diversity. Air particle counts are physically unpredictable and add an independent randomness channel. |
| Added BMP180 temperature/pressure sensor | Provides environmental entropy. Temperature fluctuations inside the chaotic box contribute additional unpredictability. |
| Built 45×45 cm chaotic box with mirrored walls | Creates a controlled but unpredictable environment — motor-driven LEDs, fan-blown particles — to maximize sensor variance. |
| Used DAC pin 25 → Gate resistor (82Ω) → MOSFET Gate | The MYOSA DAC output drives the MOSFET gate with a random voltage (0–255), creating variable drain current and noise. |

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

5. Start the Rust backend:

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
* **Rust** - Backend with SHA-256 whitening, ChaCha20 DRBG, health monitoring, and HTTP API
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
/tu-ankaja
  ├── assets/
  │   ├── box-exterior-front.png     # Chaotic box front view
  │   ├── box-exterior-side.png      # Chaotic box side view (door open)
  │   ├── mosfet-circuit-closeup.jpeg # MOSFET noise circuit on MYOSA board
  │   ├── chaotic-box-inside-full.jpeg
  │   ├── chaotic-box-fan.jpeg
  │   ├── chaotic-box-sensors.jpeg
  │   ├── chaotic-box-bottom.jpeg
  │   └── box-dimensions.jpg         # Hand-drawn design sketch
  │
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
  │   │   ├── entropy/page.tsx    # Cryptographic dashboard
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
